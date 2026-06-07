package com.gidy.client.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.gidy.client.R
import com.gidy.client.data.GidyConfig
import com.gidy.client.ui.components.AppleCard
import com.gidy.client.ui.components.AppleSwitch
import com.gidy.client.ui.components.SegmentedToggle
import com.gidy.client.ui.theme.GidyMono

private const val APP_VERSION = "v0.3.2"

@Composable
fun UserSettingsScreen(
    config: GidyConfig,
    onSave: (GidyConfig) -> Unit,
) {
    var draft by remember(config) { mutableStateOf(config) }

    LaunchedEffect(draft) {
        // Persist immediately for theme/language so changes feel native
        if (draft.theme != config.theme || draft.language != config.language) {
            onSave(draft)
        }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(horizontal = 16.dp, vertical = 8.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        AppleCard {
            Text(
                text = stringResource(R.string.set_basic),
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Spacer(Modifier.height(8.dp))
            SwitchRow(
                label = stringResource(R.string.set_auto_start),
                checked = draft.autoStart,
                onChange = { draft = draft.copy(autoStart = it) },
            )
            DividerInCard()
            SwitchRow(
                label = stringResource(R.string.set_auto_connect),
                checked = draft.autoConnect,
                onChange = { draft = draft.copy(autoConnect = it) },
            )
            DividerInCard()
            SwitchRow(
                label = stringResource(R.string.set_keep_alive),
                checked = draft.keepScreenOn,
                onChange = { draft = draft.copy(keepScreenOn = it) },
            )
            DividerInCard()
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(vertical = 10.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = stringResource(R.string.set_log_retention),
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                Text(
                    text = "${draft.logRetentionDays} ${stringResource(R.string.set_days)}",
                    style = MaterialTheme.typography.bodyMedium.copy(
                        fontFamily = GidyMono.fontFamily,
                        fontFeatureSettings = "tnum",
                    ),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }

        AppleCard {
            Text(
                text = stringResource(R.string.set_appearance),
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Spacer(Modifier.height(12.dp))
            Text(
                text = stringResource(R.string.set_theme),
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(Modifier.height(6.dp))
            SegmentedToggle(
                options = listOf(
                    "system" to stringResource(R.string.set_theme_system),
                    "light" to stringResource(R.string.set_theme_light),
                    "dark" to stringResource(R.string.set_theme_dark),
                ),
                selected = draft.theme,
                onSelect = { draft = draft.copy(theme = it) },
                modifier = Modifier.fillMaxWidth(),
            )
            Spacer(Modifier.height(14.dp))
            Text(
                text = stringResource(R.string.set_language),
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(Modifier.height(6.dp))
            SegmentedToggle(
                options = listOf(
                    "system" to "Auto",
                    "zh" to "中文",
                    "en" to "English",
                ),
                selected = draft.language,
                onSelect = { draft = draft.copy(language = it) },
                modifier = Modifier.fillMaxWidth(),
            )
        }

        AppleCard {
            Text(
                text = stringResource(R.string.set_update_check),
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Spacer(Modifier.height(10.dp))
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Text(
                    text = stringResource(R.string.set_current_version),
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Text(
                    text = APP_VERSION,
                    style = MaterialTheme.typography.bodyMedium.copy(
                        fontFamily = GidyMono.fontFamily,
                    ),
                    color = MaterialTheme.colorScheme.onSurface,
                )
            }
            Spacer(Modifier.height(12.dp))
            OutlinedSecondaryButton(
                text = stringResource(R.string.set_check_update),
                onClick = { /* no-op in UI shell */ },
            )
        }

        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.End,
        ) {
            PrimaryButton(text = stringResource(R.string.set_save)) {
                onSave(draft)
            }
        }
    }
}

@Composable
private fun SwitchRow(label: String, checked: Boolean, onChange: (Boolean) -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 10.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurface,
            fontWeight = FontWeight.Normal,
        )
        AppleSwitch(checked = checked, onCheckedChange = onChange)
    }
}

@Composable
private fun DividerInCard() {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(0.5.dp)
            .background(MaterialTheme.colorScheme.outline.copy(alpha = 0.6f)),
    )
}

@Composable
private fun PrimaryButton(text: String, onClick: () -> Unit) {
    val shape = RoundedCornerShape(12.dp)
    Box(
        modifier = Modifier
            .clip(shape)
            .background(MaterialTheme.colorScheme.primary)
            .clickable { onClick() }
            .padding(horizontal = 22.dp, vertical = 12.dp),
    ) {
        Text(
            text = text,
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onPrimary,
        )
    }
}

@Composable
private fun OutlinedSecondaryButton(text: String, onClick: () -> Unit) {
    val shape = RoundedCornerShape(12.dp)
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clip(shape)
            .background(MaterialTheme.colorScheme.surfaceVariant)
            .clickable { onClick() }
            .padding(vertical = 12.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = text,
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurface,
        )
    }
}
