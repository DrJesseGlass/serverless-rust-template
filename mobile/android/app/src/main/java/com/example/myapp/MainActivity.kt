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
import com.example.myapp.ui.theme.TrialstreamTheme
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.myapp.*
import java.net.HttpURLConnection
import java.net.URL

class MainActivity : ComponentActivity() {
    
    companion object {
        private const val TAG = "myapp"
        private const val REDIRECT_URI = "myapp://auth"
        
        // Your Cognito config - update these!
        private const val API_URL = ""
        private const val COGNITO_DOMAIN = ""
        private const val COGNITO_CLIENT_ID = ""
    }
    
    private var authCode: String? by mutableStateOf(null)
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        
        // Initialize the Rust core
        initialize(ApiConfig(
            apiUrl = API_URL,
            cognitoDomain = COGNITO_DOMAIN,
            cognitoClientId = COGNITO_CLIENT_ID
        ))
        
        // Handle OAuth redirect if launched with auth code
        handleIntent(intent)
        
        setContent {
            TrialstreamTheme {
                MainScreen(
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
        if (uri != null && uri.scheme == "trialstream" && uri.host == "auth") {
            val code = uri.getQueryParameter("code")
            if (code != null) {
                Log.d(TAG, "Received auth code")
                authCode = code
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
    }
}

@Composable
fun MainScreen(
    onLoginClick: () -> Unit,
    onLogoutClick: () -> Unit
) {
    var isAuthenticated by remember { mutableStateOf(isAuthenticated()) }
    var user by remember { mutableStateOf<User?>(null) }
    var isLoading by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    
    val scope = rememberCoroutineScope()
    
    // Check auth state
    LaunchedEffect(Unit) {
        isAuthenticated = isAuthenticated()
        if (isAuthenticated) {
            try {
                user = getCurrentUser()
            } catch (e: Exception) {
                Log.e("Trialstream", "Failed to get user", e)
            }
        }
    }
    
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
                text = "Trialstream",
                style = MaterialTheme.typography.headlineLarge
            )
            
            Spacer(modifier = Modifier.height(32.dp))
            
            if (isLoading) {
                CircularProgressIndicator()
            } else if (isAuthenticated && user != null) {
                // Logged in state
                Text(
                    text = "Welcome!",
                    style = MaterialTheme.typography.headlineSmall
                )
                Spacer(modifier = Modifier.height(8.dp))
                Text(
                    text = user?.email ?: "Unknown",
                    style = MaterialTheme.typography.bodyLarge
                )
                Spacer(modifier = Modifier.height(24.dp))
                Button(onClick = {
                    onLogoutClick()
                    isAuthenticated = false
                    user = null
                }) {
                    Text("Sign Out")
                }
            } else {
                // Logged out state
                Text(
                    text = "Sign in to continue",
                    style = MaterialTheme.typography.bodyLarge
                )
                Spacer(modifier = Modifier.height(24.dp))
                Button(onClick = onLoginClick) {
                    Text("Sign in with Google")
                }
            }
            
            error?.let {
                Spacer(modifier = Modifier.height(16.dp))
                Text(
                    text = it,
                    color = MaterialTheme.colorScheme.error
                )
            }
        }
    }
}
