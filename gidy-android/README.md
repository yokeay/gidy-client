# gidy-android

Android client (Kotlin + Jetpack Compose, Material 3) — UI shell only.

## Stack
- Kotlin 2.0 / Compose BOM 2024.10
- Material 3 with custom Apple-like monochrome theme
- DataStore Preferences for local config
- Navigation Compose with bottom navigation bar
- minSdk 26, targetSdk 34, compileSdk 34

## Status
**UI shell only.** No real proxy logic, no Rust core binding, no VPNService — those are tracked as separate tasks in the root `plan.md`.

All runtime stats are mock data for UI demonstration.

## Build (requires Android SDK + JDK 17)
```bash
cd gidy-android
./gradlew :app:assembleDebug    # generates wrapper on first run via IDE / `gradle wrapper`
```
If you don't have a Gradle wrapper jar locally, open this folder in Android Studio (Hedgehog or newer) and let it sync — the wrapper will be generated automatically.

## Package
`com.gidy.client`
