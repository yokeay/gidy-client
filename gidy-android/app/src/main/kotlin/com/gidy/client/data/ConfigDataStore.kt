package com.gidy.client.data

import android.content.Context
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.serialization.json.Json

private val Context.dataStore by preferencesDataStore(name = "gidy_config")
private val CONFIG_KEY = stringPreferencesKey("config_json")

class ConfigRepository(private val context: Context) {
    private val json = Json { ignoreUnknownKeys = true; encodeDefaults = true }

    val configFlow: Flow<GidyConfig> = context.dataStore.data.map { prefs ->
        decode(prefs[CONFIG_KEY])
    }

    suspend fun save(config: GidyConfig) {
        context.dataStore.edit { prefs ->
            prefs[CONFIG_KEY] = json.encodeToString(GidyConfig.serializer(), config)
        }
    }

    private fun decode(raw: String?): GidyConfig =
        if (raw.isNullOrBlank()) GidyConfig()
        else runCatching { json.decodeFromString(GidyConfig.serializer(), raw) }
            .getOrElse { GidyConfig() }

    @Suppress("unused")
    private fun unused(p: Preferences) = p
}
