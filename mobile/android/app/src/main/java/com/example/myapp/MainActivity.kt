package com.example.myapp

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.browser.customtabs.CustomTabsIntent
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.example.myapp.ui.theme.myappTheme
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.myapp.*
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL
import java.net.URLEncoder

class MainActivity : ComponentActivity() {
    
    companion object {
        private const val TAG = "myapp"
        private const val REDIRECT_URI = "myapp://auth"
        private const val API_URL = "https://kt3jbe9ag3.execute-api.us-east-1.amazonaws.com"
        private const val COGNITO_DOMAIN = "https://myapp-dev-021891593136.auth.us-east-1.amazoncognito.com"
        private const val COGNITO_CLIENT_ID = "2md4vst22p244mt431can1dvvf"
    }
    
    private var isLoggedIn by mutableStateOf(false)
    private var userName by mutableStateOf<String?>(null)
    private var userEmail by mutableStateOf<String?>(null)
    private var isLoading by mutableStateOf(false)
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        
        // Initialize the Rust core
        initialize(ApiConfig(
            apiUrl = API_URL,
            cognitoDomain = COGNITO_DOMAIN,
            cognitoClientId = COGNITO_CLIENT_ID
        ))
        
        // Check if already authenticated
        if (isAuthenticated()) {
            try {
                val user = getCurrentUser()
                userName = user.name
                userEmail = user.email
                isLoggedIn = true
            } catch (e: Exception) {
                Log.e(TAG, "Failed to get user", e)
            }
        }
        
        handleIntent(intent)
        
        setContent {
            myappTheme {
                MainScreen(
                    isLoggedIn = isLoggedIn,
                    userName = userName,
                    userEmail = userEmail,
                    isLoading = isLoading,
                    onLoginClick = { startOAuthFlow() },
                    onLogoutClick = { logout() }
                )
            }
        }
    }
    
    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        handleIntent(intent)
    }
    
    private fun handleIntent(intent: Intent?) {
        val uri = intent?.data
        if (uri != null && uri.scheme == "myapp" && uri.host == "auth") {
            val code = uri.getQueryParameter("code")
            if (code != null) {
                Log.d(TAG, "Received auth code")
                exchangeCodeForTokens(code)
            }
        }
    }
    
    private fun exchangeCodeForTokens(code: String) {
        isLoading = true
        
        kotlinx.coroutines.GlobalScope.launch {
            try {
                val tokenEndpoint = getTokenEndpoint()
                val tokenUrl = URL(tokenEndpoint)
                val postData = "grant_type=authorization_code" +
                    "&client_id=${URLEncoder.encode(COGNITO_CLIENT_ID, "UTF-8")}" +
                    "&code=${URLEncoder.encode(code, "UTF-8")}" +
                    "&redirect_uri=${URLEncoder.encode(REDIRECT_URI, "UTF-8")}"
                
                val connection = withContext(Dispatchers.IO) {
                    (tokenUrl.openConnection() as HttpURLConnection).apply {
                        requestMethod = "POST"
                        setRequestProperty("Content-Type", "application/x-www-form-urlencoded")
                        doOutput = true
                        outputStream.write(postData.toByteArray())
                    }
                }
                
                val response = withContext(Dispatchers.IO) {
                    connection.inputStream.bufferedReader().readText()
                }
                
                Log.d(TAG, "Token response received")
                val json = JSONObject(response)
                
                val expiresIn = json.optLong("expires_in", 3600)
                val expiresAt = (System.currentTimeMillis() / 1000) + expiresIn
                
                // Store tokens in Rust core
                setAuthTokens(AuthTokens(
                    accessToken = json.getString("access_token"),
                    idToken = json.getString("id_token"),
                    refreshToken = json.optString("refresh_token", null),
                    expiresAt = expiresAt.toULong()
                ))
                
                // Get user from Rust core
                val user = getCurrentUser()
                
                withContext(Dispatchers.Main) {
                    userName = user.name
                    userEmail = user.email
                    isLoggedIn = true
                    isLoading = false
                }
                
                Log.d(TAG, "Logged in as: $userName ($userEmail)")
            } catch (e: Exception) {
                Log.e(TAG, "Token exchange failed", e)
                withContext(Dispatchers.Main) {
                    isLoading = false
                }
            }
        }
    }
    
    private fun startOAuthFlow() {
        try {
            val authUrl = getAuthUrl(REDIRECT_URI)
            Log.d(TAG, "Opening auth URL: $authUrl")
            
            val customTabsIntent = CustomTabsIntent.Builder().build()
            customTabsIntent.launchUrl(this, Uri.parse(authUrl))
        } catch (e: Exception) {
            Log.e(TAG, "Failed to start OAuth flow", e)
        }
    }
    
    private fun logout() {
        clearAuth()
        isLoggedIn = false
        userName = null
        userEmail = null
    }
}

@Composable
fun MainScreen(
    isLoggedIn: Boolean,
    userName: String?,
    userEmail: String?,
    isLoading: Boolean,
    onLoginClick: () -> Unit,
    onLogoutClick: () -> Unit
) {
    Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .padding(24.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center
        ) {
            Text(
                text = "myapp",
                style = MaterialTheme.typography.headlineLarge
            )
            
            Spacer(modifier = Modifier.height(32.dp))
            
            when {
                isLoading -> {
                    CircularProgressIndicator()
                    Spacer(modifier = Modifier.height(16.dp))
                    Text("Signing in...")
                }
                isLoggedIn -> {
                    Text(
                        text = "Welcome!",
                        style = MaterialTheme.typography.headlineSmall
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    userName?.let {
                        Text(
                            text = it,
                            style = MaterialTheme.typography.titleMedium
                        )
                    }
                    userEmail?.let {
                        Text(
                            text = it,
                            style = MaterialTheme.typography.bodyLarge
                        )
                    }
                    Spacer(modifier = Modifier.height(24.dp))
                    Button(onClick = onLogoutClick) {
                        Text("Sign Out")
                    }
                }
                else -> {
                    Text(
                        text = "Sign in to continue",
                        style = MaterialTheme.typography.bodyLarge
                    )
                    Spacer(modifier = Modifier.height(24.dp))
                    Button(onClick = onLoginClick) {
                        Text("Sign in with Google")
                    }
                }
            }
        }
    }
}
