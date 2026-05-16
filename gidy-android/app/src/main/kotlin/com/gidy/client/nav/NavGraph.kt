package com.gidy.client.nav

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Dashboard
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material.icons.outlined.QueryStats
import androidx.compose.material.icons.outlined.Tune
import androidx.compose.material.icons.outlined.Person
import androidx.compose.ui.graphics.vector.ImageVector

enum class Destination(
    val route: String,
    val labelRes: Int,
    val icon: ImageVector,
) {
    Dashboard("dashboard", com.gidy.client.R.string.nav_dashboard, Icons.Outlined.Dashboard),
    Traffic("traffic", com.gidy.client.R.string.nav_traffic, Icons.Outlined.QueryStats),
    Config("config", com.gidy.client.R.string.nav_config, Icons.Outlined.Tune),
    Settings("settings", com.gidy.client.R.string.nav_settings, Icons.Outlined.Person),
    About("about", com.gidy.client.R.string.nav_about, Icons.Outlined.Info),
}
