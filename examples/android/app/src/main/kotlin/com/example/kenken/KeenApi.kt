package com.example.kenken

/**
 * Wrapper around UniFFI-generated Rust bindings for KenKen solver.
 *
 * This class provides a Kotlin-friendly interface to the core Rust solver functions.
 */
object KeenApi {
    init {
        // Load the native Rust library compiled via cargo-ndk
        System.loadLibrary("kenken_uniffi")
    }

    /**
     * Deduction tier controlling solver strength.
     */
    enum class DeductionTier {
        NONE, EASY, NORMAL, HARD
    }

    /**
     * Solved puzzle grid.
     */
    data class Grid(val n: Int, val cells: List<Int>) {
        fun toDisplayString(): String {
            val sb = StringBuilder()
            for (row in 0 until n) {
                for (col in 0 until n) {
                    if (col > 0) sb.append(" ")
                    sb.append(cells[row * n + col])
                }
                if (row < n - 1) sb.append("\n")
            }
            return sb.toString()
        }
    }

    /**
     * Generated puzzle with solution.
     */
    data class Generated(val desc: String, val solution: Grid)

    /**
     * Solve a puzzle from sgt-desc format.
     *
     * @param n Grid size (2-9)
     * @param desc SGT-desc format string
     * @param tier Deduction tier to use
     * @return Solved grid, or null if no solution exists
     */
    external fun solveSgtDesc(n: Int, desc: String, tier: DeductionTier): Grid?

    /**
     * Generate a random puzzle.
     *
     * @param n Grid size
     * @param seed Random seed
     * @param tier Difficulty tier
     * @return Generated puzzle with solution, or null if generation fails
     */
    external fun generateSgtDesc(n: Int, seed: Long, tier: DeductionTier): Generated?

    /**
     * Count solutions up to limit.
     *
     * Useful for verifying uniqueness: use limit=2 and check if count==1.
     *
     * @param n Grid size
     * @param desc SGT-desc format string
     * @param tier Deduction tier
     * @param limit Maximum solutions to count
     * @return Number of solutions found (up to limit)
     */
    external fun countSolutionsSgtDesc(
        n: Int,
        desc: String,
        tier: DeductionTier,
        limit: Int,
    ): Int
}
