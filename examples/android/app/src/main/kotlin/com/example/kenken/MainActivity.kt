package com.example.kenken

import android.os.Bundle
import android.view.KeyEvent
import android.widget.Button
import android.widget.EditText
import android.widget.RadioButton
import android.widget.RadioGroup
import android.widget.ScrollView
import android.widget.Spinner
import android.widget.TextView
import androidx.activity.viewModels
import androidx.appcompat.app.AppCompatActivity
import androidx.core.view.isVisible

/**
 * Main activity for the KenKen solver example app.
 *
 * Provides a simple UI for:
 * - Entering grid size and puzzle description
 * - Selecting solver deduction tier
 * - Displaying solutions
 */
class MainActivity : AppCompatActivity() {
    private val viewModel: PuzzleViewModel by viewModels()

    private lateinit var gridSizeSpinner: Spinner
    private lateinit var puzzleDescInput: EditText
    private lateinit var tierRadioGroup: RadioGroup
    private lateinit var solveButton: Button
    private lateinit var exampleButton: Button
    private lateinit var solutionOutput: TextView
    private lateinit var errorOutput: TextView
    private lateinit var loadingOutput: TextView
    private lateinit var statusOutput: TextView

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        // Bind UI elements
        gridSizeSpinner = findViewById(R.id.gridSizeSpinner)
        puzzleDescInput = findViewById(R.id.puzzleDescInput)
        tierRadioGroup = findViewById(R.id.tierRadioGroup)
        solveButton = findViewById(R.id.solveButton)
        exampleButton = findViewById(R.id.exampleButton)
        solutionOutput = findViewById(R.id.solutionOutput)
        errorOutput = findViewById(R.id.errorOutput)
        loadingOutput = findViewById(R.id.loadingOutput)
        statusOutput = findViewById(R.id.statusOutput)

        setupUI()
        observeViewModel()
    }

    private fun setupUI() {
        // Grid size spinner
        gridSizeSpinner.setSelection(1) // Default 3x3

        // Puzzle input
        puzzleDescInput.setOnKeyListener { _, keyCode, event ->
            if (keyCode == KeyEvent.KEYCODE_ENTER && event.action == KeyEvent.ACTION_UP) {
                viewModel.solve()
                true
            } else {
                false
            }
        }

        // Tier selection (default: Normal)
        tierRadioGroup.check(R.id.tierNormal)

        // Solve button
        solveButton.setOnClickListener {
            updateViewModelFromUI()
            viewModel.solve()
        }

        // Example button - loads a sample puzzle
        exampleButton.setOnClickListener {
            puzzleDescInput.setText(PuzzleViewModel.EXAMPLE_2X2)
            gridSizeSpinner.setSelection(0) // 2x2
            tierRadioGroup.check(R.id.tierNormal)
            updateViewModelFromUI()
            viewModel.solve()
        }
    }

    private fun updateViewModelFromUI() {
        // Grid size: spinner contains values 2-9
        val sizeIndex = gridSizeSpinner.selectedItemPosition
        val size = sizeIndex + 2
        viewModel.setGridSize(size)

        // Puzzle description
        val desc = puzzleDescInput.text.toString()
        viewModel.setPuzzleDesc(desc)

        // Deduction tier
        val tier = when (tierRadioGroup.checkedRadioButtonId) {
            R.id.tierNone -> KeenApi.DeductionTier.NONE
            R.id.tierEasy -> KeenApi.DeductionTier.EASY
            R.id.tierNormal -> KeenApi.DeductionTier.NORMAL
            R.id.tierHard -> KeenApi.DeductionTier.HARD
            else -> KeenApi.DeductionTier.NORMAL
        }
        viewModel.setDeductionTier(tier)
    }

    private fun observeViewModel() {
        // Solution display
        viewModel.solution.observe(this) { solution ->
            if (solution != null) {
                solutionOutput.text = buildString {
                    append("Solution (${solution.n}x${solution.n}):\n")
                    append("─".repeat(solution.n * 2))\n")
                    append(solution.toDisplayString())
                }
                solutionOutput.isVisible = true
            } else {
                solutionOutput.isVisible = false
            }
        }

        // Error display
        viewModel.error.observe(this) { error ->
            if (error != null) {
                errorOutput.text = "❌ Error: $error"
                errorOutput.isVisible = true
            } else {
                errorOutput.isVisible = false
            }
        }

        // Loading state
        viewModel.isLoading.observe(this) { isLoading ->
            if (isLoading) {
                loadingOutput.text = "⏳ Solving..."
                loadingOutput.isVisible = true
                solveButton.isEnabled = false
            } else {
                loadingOutput.isVisible = false
                solveButton.isEnabled = true
            }
        }

        // Uniqueness check
        viewModel.uniqueness.observe(this) { isUnique ->
            if (isUnique != null) {
                statusOutput.text = if (isUnique) {
                    "✓ Puzzle is unique (1 solution)"
                } else {
                    "⚠ Puzzle has multiple solutions"
                }
                statusOutput.isVisible = true
            } else {
                statusOutput.isVisible = false
            }
        }
    }
}
