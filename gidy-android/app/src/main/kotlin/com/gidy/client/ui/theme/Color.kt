package com.gidy.client.ui.theme

import androidx.compose.material3.ColorScheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.ui.graphics.Color

object GidyPalette {
    // Light (zinc-50/100/200/500/900)
    val LightBg = Color(0xFFFAFAFA)
    val LightSurface = Color(0xFFFFFFFF)
    val LightSurfaceVariant = Color(0xFFF4F4F5)
    val LightOutline = Color(0xFFE4E4E7)
    val LightOnSurface = Color(0xFF18181B)
    val LightMutedFg = Color(0xFF71717A)
    val LightPrimary = Color(0xFF18181B)
    val LightOnPrimary = Color(0xFFFAFAFA)

    // Dark
    val DarkBg = Color(0xFF09090B)
    val DarkSurface = Color(0xFF111113)
    val DarkSurfaceVariant = Color(0xFF1C1C1F)
    val DarkOutline = Color(0xFF27272A)
    val DarkOnSurface = Color(0xFFFAFAFA)
    val DarkMutedFg = Color(0xFFA1A1AA)
    val DarkPrimary = Color(0xFFFAFAFA)
    val DarkOnPrimary = Color(0xFF09090B)

    val Destructive = Color(0xFFEF4444)
}

val LightColors: ColorScheme = lightColorScheme(
    primary = GidyPalette.LightPrimary,
    onPrimary = GidyPalette.LightOnPrimary,
    secondary = GidyPalette.LightPrimary,
    onSecondary = GidyPalette.LightOnPrimary,
    background = GidyPalette.LightBg,
    onBackground = GidyPalette.LightOnSurface,
    surface = GidyPalette.LightSurface,
    onSurface = GidyPalette.LightOnSurface,
    surfaceVariant = GidyPalette.LightSurfaceVariant,
    onSurfaceVariant = GidyPalette.LightMutedFg,
    outline = GidyPalette.LightOutline,
    outlineVariant = GidyPalette.LightOutline,
    error = GidyPalette.Destructive,
    onError = GidyPalette.LightOnPrimary,
)

val DarkColors: ColorScheme = darkColorScheme(
    primary = GidyPalette.DarkPrimary,
    onPrimary = GidyPalette.DarkOnPrimary,
    secondary = GidyPalette.DarkPrimary,
    onSecondary = GidyPalette.DarkOnPrimary,
    background = GidyPalette.DarkBg,
    onBackground = GidyPalette.DarkOnSurface,
    surface = GidyPalette.DarkSurface,
    onSurface = GidyPalette.DarkOnSurface,
    surfaceVariant = GidyPalette.DarkSurfaceVariant,
    onSurfaceVariant = GidyPalette.DarkMutedFg,
    outline = GidyPalette.DarkOutline,
    outlineVariant = GidyPalette.DarkOutline,
    error = GidyPalette.Destructive,
    onError = GidyPalette.DarkOnPrimary,
)
