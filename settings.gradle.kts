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
    versionCatalogs {
        create("libs") {
            from(files("fga/gradle/libs.versions.toml"))
        }
    }
}

rootProject.name = "fga-apple"

include(":libautomata")
project(":libautomata").projectDir = file("fga/libautomata")
include(":scripts")
project(":scripts").projectDir = file("fga/scripts")

include(":host")
