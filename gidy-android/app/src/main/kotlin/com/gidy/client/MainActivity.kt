package com.gidy.client

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.gidy.client.data.ConfigRepository
import com.gidy.client.nav.Destination
import com.gidy.client.ui.screens.AboutScreen
import com.gidy.client.ui.screens.DashboardScreen
import com.gidy.client.ui.screens.SystemConfigScreen
import com.gidy.client.ui.screens.TrafficMonitorScreen
import com.gidy.client.ui.screens.UserSettingsScreen
import com.gidy.client.ui.theme.GidyTheme
import com.gidy.client.ui.theme.ThemePref
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent { GidyRoot() }
    }
}

@Composable
private fun GidyRoot() {
    val context = LocalContext.current
    val repo = remember { ConfigRepository(context.applicationContext) }
    val scope = rememberCoroutineScope()

    val config by repo.configFlow.collectAsState(initial = com.gidy.client.data.GidyConfig())

    val themePref = when (config.theme) {
        "light" -> ThemePref.Light
        "dark" -> ThemePref.Dark
        else -> ThemePref.System
    }

    GidyTheme(pref = themePref) {
        val nav = rememberNavController()
        val backStack by nav.currentBackStackEntryAsState()
        val currentRoute = backStack?.destination?.route

        Scaffold(
            containerColor = MaterialTheme.colorScheme.background,
            topBar = {
                val titleRes = Destination.values()
                    .firstOrNull { it.route == currentRoute }
                    ?.labelRes
                    ?: R.string.app_name
                TopBar(title = stringResource(titleRes))
            },
            bottomBar = {
                BottomBar(currentRoute = currentRoute) { dest ->
                    nav.navigate(dest.route) {
                        popUpTo(nav.graph.startDestinationId) { saveState = true }
                        launchSingleTop = true
                        restoreState = true
                    }
                }
            },
        ) { inner ->
            NavHost(
                navController = nav,
                startDestination = Destination.Dashboard.route,
                modifier = Modifier
                    .padding(inner)
                    .fillMaxSize(),
            ) {
                composable(Destination.Dashboard.route) {
                    DashboardScreen(config = config)
                }
                composable(Destination.Traffic.route) {
                    TrafficMonitorScreen()
                }
                composable(Destination.Config.route) {
                    SystemConfigScreen(
                        config = config,
                        onSave = { newCfg -> scope.launch { repo.save(newCfg) } },
                    )
                }
                composable(Destination.Settings.route) {
                    UserSettingsScreen(
                        config = config,
                        onSave = { newCfg -> scope.launch { repo.save(newCfg) } },
                    )
                }
                composable(Destination.About.route) {
                    AboutScreen()
                }
            }
        }
    }

}

@Composable
private fun TopBar(title: String) {
    Surface(color = MaterialTheme.colorScheme.background, modifier = Modifier.fillMaxWidth()) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 20.dp, vertical = 14.dp),
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.headlineLarge.copy(
                    fontWeight = FontWeight.SemiBold,
                    fontSize = 28.sp,
                ),
                color = MaterialTheme.colorScheme.onSurface,
            )
        }
    }
}

@Composable
private fun BottomBar(
    currentRoute: String?,
    onNavigate: (Destination) -> Unit,
) {
    val shape = RoundedCornerShape(22.dp)
    Surface(
        color = MaterialTheme.colorScheme.background,
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 12.dp, vertical = 8.dp),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .clip(shape)
                .background(MaterialTheme.colorScheme.surface)
                .padding(vertical = 8.dp, horizontal = 4.dp),
            horizontalArrangement = Arrangement.SpaceEvenly,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Destination.values().forEach { dest ->
                val selected = currentRoute == dest.route ||
                    (currentRoute == null && dest == Destination.Dashboard)
                val tint = if (selected)
                    MaterialTheme.colorScheme.onSurface
                else
                    MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.55f)

                val interaction = remember { MutableInteractionSource() }
                Column(
                    horizontalAlignment = Alignment.CenterHorizontally,
                    modifier = Modifier
                        .weight(1f)
                        .clip(RoundedCornerShape(12.dp))
                        .clickable(
                            interactionSource = interaction,
                            indication = null,
                        ) { onNavigate(dest) }
                        .padding(vertical = 6.dp),
                ) {
                    Icon(
                        imageVector = dest.icon,
                        contentDescription = stringResource(dest.labelRes),
                        modifier = Modifier.size(22.dp),
                        tint = tint,
                    )
                    Spacer(Modifier.height(3.dp))
                    Text(
                        text = stringResource(dest.labelRes),
                        style = MaterialTheme.typography.labelSmall,
                        color = tint,
                    )
                }
            }
        }
    }
}
