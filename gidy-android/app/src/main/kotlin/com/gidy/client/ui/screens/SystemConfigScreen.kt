package com.gidy.client.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.compose.foundation.clickable
import com.gidy.client.R
import com.gidy.client.data.GidyConfig
import com.gidy.client.ui.components.AppleCard
import com.gidy.client.ui.components.SegmentedToggle
import com.gidy.client.ui.components.StatusBadge
import com.gidy.client.ui.theme.GidyMono
import kotlin.random.Random

@Composable
fun SystemConfigScreen(
    config: GidyConfig,
    onSave: (GidyConfig) -> Unit,
) {
    var draft by remember(config) { mutableStateOf(config) }
    var savedTick by remember { mutableStateOf(0) }
    val running = false // UI shell — proxy logic not wired

    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(horizontal = 16.dp, vertical = 8.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.End,
        ) {
            StatusBadge(
                connected = running,
                label = if (running)
                    stringResource(R.string.common_connected)
                else
                    stringResource(R.string.common_disconnected),
            )
        }

        AppleCard {
            Text(
                text = stringResource(R.string.cfg_proxy_server),
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Spacer(Modifier.height(14.dp))
            LabeledField(
                label = stringResource(R.string.cfg_addr),
                value = draft.serverAddr,
                onChange = { draft = draft.copy(serverAddr = it) },
            )
            Spacer(Modifier.height(10.dp))
            LabeledField(
                label = stringResource(R.string.cfg_port),
                value = draft.serverPort.toString(),
                onChange = { draft = draft.copy(serverPort = it.toIntOrNull() ?: draft.serverPort) },
                keyboard = KeyboardType.Number,
            )
            Spacer(Modifier.height(10.dp))
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
                modifier = Modifier.fillMaxWidth(),
            ) {
                Text(
                    text = stringResource(R.string.cfg_psk),
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Text(
                    text = stringResource(R.string.cfg_generate_psk),
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                    modifier = Modifier.clickable {
                        draft = draft.copy(pskHex = generatePsk())
                    },
                )
            }
            Spacer(Modifier.height(6.dp))
            Field(
                value = draft.pskHex,
                onChange = { draft = draft.copy(pskHex = it) },
                placeholder = stringResource(R.string.cfg_psk_hint),
            )
        }

        AppleCard {
            Text(
                text = stringResource(R.string.cfg_local_proxy),
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Spacer(Modifier.height(14.dp))
            LabeledField(
                label = "${stringResource(R.string.cfg_socks5)} ${stringResource(R.string.cfg_addr)}",
                value = draft.socks5Addr,
                onChange = { draft = draft.copy(socks5Addr = it) },
            )
            Spacer(Modifier.height(10.dp))
            LabeledField(
                label = "${stringResource(R.string.cfg_socks5)} ${stringResource(R.string.cfg_port)}",
                value = draft.socks5Port.toString(),
                onChange = { draft = draft.copy(socks5Port = it.toIntOrNull() ?: draft.socks5Port) },
                keyboard = KeyboardType.Number,
            )
            Spacer(Modifier.height(10.dp))
            LabeledField(
                label = "${stringResource(R.string.cfg_http)} ${stringResource(R.string.cfg_addr)}",
                value = draft.httpAddr,
                onChange = { draft = draft.copy(httpAddr = it) },
            )
            Spacer(Modifier.height(10.dp))
            LabeledField(
                label = "${stringResource(R.string.cfg_http)} ${stringResource(R.string.cfg_port)}",
                value = draft.httpPort.toString(),
                onChange = { draft = draft.copy(httpPort = it.toIntOrNull() ?: draft.httpPort) },
                keyboard = KeyboardType.Number,
            )
            Spacer(Modifier.height(14.dp))
            Text(
                text = stringResource(R.string.cfg_mode),
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(Modifier.height(8.dp))
            SegmentedToggle(
                options = listOf(
                    "global" to stringResource(R.string.cfg_mode_global),
                    "pac" to stringResource(R.string.cfg_mode_pac),
                ),
                selected = draft.routingMode,
                onSelect = { draft = draft.copy(routingMode = it) },
                modifier = Modifier.fillMaxWidth(),
            )
        }

        AppleCard {
            Text(
                text = stringResource(R.string.cfg_note_title),
                style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Spacer(Modifier.height(6.dp))
            Text(
                text = stringResource(R.string.cfg_note_body),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }

        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.End,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            if (savedTick > 0) {
                Text(
                    text = stringResource(R.string.cfg_saved),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(end = 12.dp),
                )
            }
            PrimaryButton(text = stringResource(R.string.cfg_save)) {
                onSave(draft)
                savedTick++
            }
        }
    }
}

@Composable
private fun LabeledField(
    label: String,
    value: String,
    onChange: (String) -> Unit,
    keyboard: KeyboardType = KeyboardType.Text,
) {
    Column {
        Text(
            text = label,
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(Modifier.height(6.dp))
        Field(value = value, onChange = onChange, keyboard = keyboard)
    }
}

@Composable
private fun Field(
    value: String,
    onChange: (String) -> Unit,
    keyboard: KeyboardType = KeyboardType.Text,
    placeholder: String? = null,
) {
    val shape = RoundedCornerShape(10.dp)
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clip(shape)
            .background(MaterialTheme.colorScheme.surfaceVariant)
            .border(0.5.dp, MaterialTheme.colorScheme.outline, shape)
            .padding(horizontal = 12.dp, vertical = 10.dp),
        contentAlignment = Alignment.CenterStart,
    ) {
        BasicTextField(
            value = value,
            onValueChange = onChange,
            singleLine = true,
            cursorBrush = SolidColor(MaterialTheme.colorScheme.onSurface),
            textStyle = TextStyle(
                color = MaterialTheme.colorScheme.onSurface,
                fontSize = MaterialTheme.typography.bodyMedium.fontSize,
                fontFamily = GidyMono.fontFamily,
                fontFeatureSettings = "tnum",
            ),
            keyboardOptions = KeyboardOptions(
                keyboardType = keyboard,
                imeAction = ImeAction.Done,
            ),
        )
        if (value.isEmpty() && placeholder != null) {
            Text(
                text = placeholder,
                style = MaterialTheme.typography.bodyMedium.copy(
                    fontFamily = GidyMono.fontFamily,
                ),
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.7f),
            )
        }
    }

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

private fun generatePsk(): String {
    val hex = "0123456789abcdef"
    val sb = StringBuilder(64)
    repeat(64) { sb.append(hex[Random.nextInt(16)]) }
    return sb.toString()
}
