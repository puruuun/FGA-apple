package io.github.fga_apple.host.imaging

/** 데스크톱용 OpenCV 네이티브 라이브러리를 한 번만 올린다. */
object OpenCv {
    private val loaded by lazy {
        nu.pattern.OpenCV.loadLocally()
        true
    }

    fun load() {
        check(loaded)
    }
}
