import Foundation
import AuthenticationServices

@MainActor
class AuthManager: NSObject, ObservableObject {
    @Published var isAuthenticated = false
    @Published var userName: String?
    @Published var userEmail: String?
    @Published var isLoading = false
    @Published var errorMessage: String?
    
    // Cognito configuration - from your terraform/Android setup
    private let cognitoDomain = ""
    private let cognitoClientId = ""
    private let apiUrl = "https://api.myapp.com"  // placeholder, not needed for login
    private let redirectUri = "myapp://auth"
    
    private var webAuthSession: ASWebAuthenticationSession?
    
    override init() {
        super.init()
        
        // Initialize the Rust core with config
        let config = ApiConfig(
            apiUrl: apiUrl,
            cognitoDomain: cognitoDomain,
            cognitoClientId: cognitoClientId
        )
        initialize(config: config)
        
        // Check if we have existing valid tokens
        if myapp.isAuthenticated() {
            self.isAuthenticated = true
            loadUserInfo()
        }
    }
    
    func startOAuthFlow() {
        isLoading = true
        errorMessage = nil
        
        do {
            let authUrlString = try getAuthUrl(redirectUri: redirectUri)
            guard let authUrl = URL(string: authUrlString) else {
                errorMessage = "Invalid auth URL"
                isLoading = false
                return
            }
            
            // Use ASWebAuthenticationSession for secure OAuth
            webAuthSession = ASWebAuthenticationSession(
                url: authUrl,
                callbackURLScheme: "myapp"
            ) { [weak self] callbackURL, error in
                Task { @MainActor in
                    self?.isLoading = false
                    
                    if let error = error as? ASWebAuthenticationSessionError,
                       error.code == .canceledLogin {
                        print("myapp: User cancelled login")
                        return
                    }
                    
                    if let error = error {
                        self?.errorMessage = "Auth error: \(error.localizedDescription)"
                        return
                    }
                    
                    guard let url = callbackURL,
                          let components = URLComponents(url: url, resolvingAgainstBaseURL: false),
                          let code = components.queryItems?.first(where: { $0.name == "code" })?.value else {
                        self?.errorMessage = "No auth code received"
                        return
                    }
                    
                    await self?.exchangeCodeForTokens(code: code)
                }
            }
            
            webAuthSession?.presentationContextProvider = self
            webAuthSession?.prefersEphemeralWebBrowserSession = false
            webAuthSession?.start()
            
        } catch {
            errorMessage = "Failed to start auth: \(error.localizedDescription)"
            isLoading = false
        }
    }
    
    func exchangeCodeForTokens(code: String) async {
        isLoading = true
        errorMessage = nil
        
        do {
            let tokenEndpoint = try getTokenEndpoint()
            guard let tokenUrl = URL(string: tokenEndpoint) else {
                errorMessage = "Invalid token endpoint"
                isLoading = false
                return
            }
            
            // Build token request (URL-encoded form data)
            var request = URLRequest(url: tokenUrl)
            request.httpMethod = "POST"
            request.setValue("application/x-www-form-urlencoded", forHTTPHeaderField: "Content-Type")
            
            let bodyParams = [
                "grant_type": "authorization_code",
                "client_id": cognitoClientId,
                "code": code,
                "redirect_uri": redirectUri
            ]
            
            let bodyString = bodyParams
                .map { "\($0.key)=\($0.value.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? $0.value)" }
                .joined(separator: "&")
            
            request.httpBody = bodyString.data(using: .utf8)
            
            let (data, response) = try await URLSession.shared.data(for: request)
            
            guard let httpResponse = response as? HTTPURLResponse else {
                errorMessage = "Invalid response"
                isLoading = false
                return
            }
            
            if httpResponse.statusCode != 200 {
                let errorBody = String(data: data, encoding: .utf8) ?? "Unknown error"
                print("myapp: Token exchange failed: \(errorBody)")
                errorMessage = "Token exchange failed (HTTP \(httpResponse.statusCode))"
                isLoading = false
                return
            }
            
            // Parse token response
            let tokenResponse = try JSONDecoder().decode(TokenResponse.self, from: data)
            
            // Store tokens in Rust core
            // expiresAt: current time + expires_in seconds (as Unix timestamp)
            let expiresAt = UInt64(Date().timeIntervalSince1970) + UInt64(tokenResponse.expires_in ?? 3600)
            
            let tokens = AuthTokens(
                accessToken: tokenResponse.access_token,
                idToken: tokenResponse.id_token,
                refreshToken: tokenResponse.refresh_token,
                expiresAt: expiresAt
            )
            setAuthTokens(tokens: tokens)
            
            isAuthenticated = true
            loadUserInfo()
            print("myapp: Login successful")
            
        } catch {
            errorMessage = "Token exchange error: \(error.localizedDescription)"
            print("myapp: Token exchange error: \(error)")
        }
        
        isLoading = false
    }
    
    func logout() {
        clearAuth()
        isAuthenticated = false
        userName = nil
        userEmail = nil
    }
    
    private func loadUserInfo() {
        do {
            let user = try getCurrentUser()
            userName = user.name
            userEmail = user.email
        } catch {
            print("myapp: Failed to get user info: \(error)")
        }
    }
}

// MARK: - ASWebAuthenticationPresentationContextProviding

extension AuthManager: ASWebAuthenticationPresentationContextProviding {
    func presentationAnchor(for session: ASWebAuthenticationSession) -> ASPresentationAnchor {
        guard let scene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
              let window = scene.windows.first else {
            return ASPresentationAnchor()
        }
        return window
    }
}

// MARK: - Token Response

private struct TokenResponse: Decodable {
    let access_token: String
    let id_token: String
    let refresh_token: String?
    let expires_in: Int?
}

// MARK: - Switch Account (forces fresh login)

extension AuthManager {
    func switchAccount() {
        // Don't clear auth yet - wait for successful login
        isLoading = true
        errorMessage = nil
        
        do {
            let authUrlString = try getAuthUrl(redirectUri: redirectUri)
            guard let authUrl = URL(string: authUrlString) else {
                errorMessage = "Invalid auth URL"
                isLoading = false
                return
            }
            
            webAuthSession = ASWebAuthenticationSession(
                url: authUrl,
                callbackURLScheme: "trialstream"
            ) { [weak self] callbackURL, error in
                Task { @MainActor in
                    self?.isLoading = false
                    
                    if let error = error as? ASWebAuthenticationSessionError,
                       error.code == .canceledLogin {
                        // User cancelled - keep existing session intact
                        return
                    }
                    
                    if let error = error {
                        self?.errorMessage = "Auth error: \(error.localizedDescription)"
                        // Keep existing session on error
                        return
                    }
                    
                    guard let url = callbackURL,
                          let components = URLComponents(url: url, resolvingAgainstBaseURL: false),
                          let code = components.queryItems?.first(where: { $0.name == "code" })?.value else {
                        self?.errorMessage = "No auth code received"
                        return
                    }
                    
                    // Clear old session only after we have a new auth code
                    clearAuth()
                    
                    await self?.exchangeCodeForTokens(code: code)
                }
            }
            
            webAuthSession?.presentationContextProvider = self
            webAuthSession?.prefersEphemeralWebBrowserSession = true
            webAuthSession?.start()
            
        } catch {
            errorMessage = "Failed to start auth: \(error.localizedDescription)"
            isLoading = false
        }
    }
}
