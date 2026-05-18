package com.gidy.client.ui.theme

import androidx.compose.material3.Typography
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp

private val systemSans = FontFamily.Default
private val systemMono = FontFamily.Monospace

val GidyTypography = Typography(
    displayLarge = TextStyle(fontFamily = systemSans, fontWeight = FontWeight.SemiBold, fontSize = 32.sp, letterSpacing = (-0.4).sp),
    displayMedium = TextStyle(fontFamily = systemSans, fontWeight = FontWeight.SemiBold, fontSize = 28.sp, letterSpacing = (-0.3).sp),
    headlineLarge = TextStyle(fontFamily = systemSans, fontWeight = FontWeight.SemiBold, fontSize = 24.sp, letterSpacing = (-0.2).sp),
    headlineMedium = TextStyle(fontFamily = systemSans, fontWeight = FontWeight.SemiBold, fontSize = 20.sp),
    headlineSmall = TextStyle(fontFamily = systemSans, fontWeight = FontWeight.SemiBold, fontSize = 17.sp),
    titleLarge = TextStyle(fontFamily = systemSans, fontWeight = FontWeight.SemiBold, fontSize = 17.sp),
    titleMedium = TextStyle(fontFamily = systemSans, fontWeight = FontWeight.Medium, fontSize = 15.sp),
    titleSmall = TextStyle(fontFamily = systemSans, fontWeight = FontWeight.Medium, fontSize = 13.sp),
    bodyLarge = TextStyle(fontFamily = systemSans, fontSize = 15.sp, lineHeight = 22.sp),
    bodyMedium = TextStyle(fontFamily = systemSans, fontSize = 14.sp, lineHeight = 20.sp),
    bodySmall = TextStyle(fontFamily = systemSans, fontSize = 12.sp, lineHeight = 16.sp),
    labelLarge = TextStyle(fontFamily = systemSans, fontWeight = FontWeight.Medium, fontSize = 14.sp),
    labelMedium = TextStyle(fontFamily = systemSans, fontWeight = FontWeight.Medium, fontSize = 12.sp, letterSpacing = 0.4.sp),
    labelSmall = TextStyle(fontFamily = systemSans, fontWeight = FontWeight.Medium, fontSize = 11.sp, letterSpacing = 0.6.sp),
)

val GidyMono = TextStyle(fontFamily = systemMono, fontFeatureSettings = "tnum")
