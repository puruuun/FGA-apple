package io.github.fga_apple.host.golden

class Xorshift(seed: Long) {
    private var state = if (seed == 0L) 0x2545F4914F6CDD1DL else seed

    fun nextLong(): Long {
        state = state xor (state ushr 12)
        state = state xor (state shl 25)
        state = state xor (state ushr 27)
        return state * 2685821657736338717L
    }

    fun nextInt(bound: Int): Int {
        require(bound > 0)
        return ((nextLong() ushr 33) % bound).toInt()
    }
}
