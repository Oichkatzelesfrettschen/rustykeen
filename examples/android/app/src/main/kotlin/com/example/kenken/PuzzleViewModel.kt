package com.example.kenken

import androidx.lifecycle.LiveData
import androidx.lifecycle.MutableLiveData
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

/**
 * ViewModel managing KenKen puzzle state and solver operations.
 *
 * Handles:
 * - Puzzle input (size and description)
 * - Solver configuration (deduction tier)
 * - Solution display
 * - Error reporting
 */
class PuzzleViewModel : ViewModel() {
    // Input state
    private val _gridSize = MutableLiveData(3)
    val gridSize: LiveData<Int> = _gridSize

    private val _puzzleDesc = MutableLiveData("")
    val puzzleDesc: LiveData<String> = _puzzleDesc

    private val _deductionTier = MutableLiveData(KeenApi.DeductionTier.NORMAL)
    val deductionTier: LiveData<KeenApi.DeductionTier> = _deductionTier

    // Output state
    private val _solution = MutableLiveData<KeenApi.Grid?>(null)
    val solution: LiveData<KeenApi.Grid?> = _solution

    private val _isLoading = MutableLiveData(false)
    val isLoading: LiveData<Boolean> = _isLoading

    private val _error = MutableLiveData<String?>(null)
    val error: LiveData<String?> = _error

    private val _uniqueness = MutableLiveData<Boolean?>(null)
    val uniqueness: LiveData<Boolean?> = _uniqueness

    fun setGridSize(size: Int) {
        _gridSize.value = size.coerceIn(2, 9)
    }

    fun setPuzzleDesc(desc: String) {
        _puzzleDesc.value = desc
    }

    fun setDeductionTier(tier: KeenApi.DeductionTier) {
        _deductionTier.value = tier
    }

    fun solve() {
        val size = _gridSize.value ?: 3
        val desc = _puzzleDesc.value ?: ""
        val tier = _deductionTier.value ?: KeenApi.DeductionTier.NORMAL

        if (desc.isBlank()) {
            _error.value = "Please enter a puzzle description"
            return
        }

        if (size < 2 || size > 9) {
            _error.value = "Grid size must be between 2 and 9"
            return
        }

        _isLoading.value = true
        _error.value = null
        _solution.value = null
        _uniqueness.value = null

        viewModelScope.launch(Dispatchers.Default) {
            try {
                val solution = KeenApi.solveSgtDesc(size, desc, tier)
                if (solution != null) {
                    _solution.postValue(solution)
                    _error.postValue(null)
                    // Optionally check uniqueness
                    checkUniqueness(size, desc, tier)
                } else {
                    _solution.postValue(null)
                    _error.postValue("No solution found. Check puzzle description.")
                }
            } catch (e: Exception) {
                _solution.postValue(null)
                _error.postValue("Error: ${e.message}")
            } finally {
                _isLoading.postValue(false)
            }
        }
    }

    private fun checkUniqueness(size: Int, desc: String, tier: KeenApi.DeductionTier) {
        try {
            // Count up to 2 solutions to determine uniqueness
            val count = KeenApi.countSolutionsSgtDesc(size, desc, tier, 2)
            _uniqueness.postValue(count == 1)
        } catch (e: Exception) {
            // Silently ignore uniqueness check errors
            _uniqueness.postValue(null)
        }
    }

    fun clearError() {
        _error.value = null
    }

    // Example puzzle for quick testing
    companion object {
        const val EXAMPLE_2X2 = "_5,a1a2a2a1"
        const val EXAMPLE_3X3 = "_13,a1a2a3a2a3a1a3a1a2"
        const val EXAMPLE_4X4 = "_25,a1a2a4a3a3a4a2a1a4a3a1a2a2a1a3a4"
    }
}
