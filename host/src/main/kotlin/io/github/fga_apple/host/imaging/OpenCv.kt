package io.github.fga_apple.host.imaging

object OpenCv {
    private val loaded by lazy {
        nu.pattern.OpenCV.loadLocally()
        true
    }

    fun load() {
        check(loaded)
    }
}
