import SwiftUI

struct ContentView: View {
    @EnvironmentObject var authManager: AuthManager
    
    var body: some View {
        VStack(spacing: 24) {
            Spacer()
            
            // Logo / Title
            Text("Trialstream")
                .font(.largeTitle)
                .fontWeight(.bold)
            
            if authManager.isAuthenticated {
                // Logged in state
                authenticatedView
            } else {
                // Login state
                loginView
            }
            
            Spacer()
        }
        .padding()
    }
    
    // MARK: - Authenticated View
    
    private var authenticatedView: some View {
        VStack(spacing: 16) {
            Image(systemName: "checkmark.circle.fill")
                .font(.system(size: 60))
                .foregroundColor(.green)
            
            Text("Welcome!")
                .font(.title2)
            
            if let name = authManager.userName {
                Text(name)
                    .font(.headline)
            }
            
            if let email = authManager.userEmail {
                Text(email)
                    .font(.subheadline)
                    .foregroundColor(.secondary)
            }
            
            Button(action: {
                authManager.logout()
            }) {
                Text("Sign Out")
                    .frame(maxWidth: .infinity)
                    .padding()
                    .background(Color.red.opacity(0.1))
                    .foregroundColor(.red)
                    .cornerRadius(10)
            }
            .padding(.top, 20)
        }
    }
    
    // MARK: - Login View
    
    private var loginView: some View {
        VStack(spacing: 16) {
            Text("Sign in to continue")
                .font(.subheadline)
                .foregroundColor(.secondary)
            
            if authManager.isLoading {
                ProgressView()
                    .scaleEffect(1.5)
                    .padding()
            } else {
                Button(action: {
                    authManager.startOAuthFlow()
                }) {
                    HStack {
                        Image(systemName: "person.circle.fill")
                        Text("Sign in with Google")
                    }
                    .frame(maxWidth: .infinity)
                    .padding()
                    .background(Color.blue)
                    .foregroundColor(.white)
                    .cornerRadius(10)
                }
            }
            
            if let error = authManager.errorMessage {
                Text(error)
                    .font(.caption)
                    .foregroundColor(.red)
                    .multilineTextAlignment(.center)
                    .padding(.top, 8)
            }
        }
    }
}

#Preview {
    ContentView()
        .environmentObject(AuthManager())
}
