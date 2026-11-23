pluginManagement {
    repositories {
        mavenCentral()
        gradlePluginPortal()
    }
}

plugins {
    id("org.gradle.toolchains.foojay-resolver-convention") version "1.0.0"
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        mavenCentral()
    }
    // FGA-Modified와 같은 라이브러리 버전을 쓰도록 그쪽 버전 카탈로그를 그대로 읽는다.
    versionCatalogs {
        create("libs") {
            from(files("fga/gradle/libs.versions.toml"))
        }
    }
}

rootProject.name = "fga-apple"

// FGA-Modified(서브모듈)의 순수 JVM 모듈 두 개를 그대로 빌드에 넣는다.
// Android 모듈(:app, :prefs)은 넣지 않으므로 Android SDK가 필요 없다.
include(":libautomata")
project(":libautomata").projectDir = file("fga/libautomata")
include(":scripts")
project(":scripts").projectDir = file("fga/scripts")

include(":host")
