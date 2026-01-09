import SwiftUI

@main
struct myappApp: App {
    @StateObject private var authManager = AuthManager()
    
    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(authManager)
                .onOpenURL { url in
                    // Handle myapp://auth?code=xxx redirect
                    handleOAuthRedirect(url: url)
                }
        }
    }
    
    private func handleOAuthRedirect(url: URL) {
        guard url.scheme == "myapp",
              url.host == "auth",
              let components = URLComponents(url: url, resolvingAgainstBaseURL: false),
              let code = components.queryItems?.first(where: { $0.name == "code" })?.value else {
            print("myapp: Invalid OAuth redirect URL: \(url)")
            return
        }
        
        print("myapp: Received auth code")
        Task {
            await authManager.exchangeCodeForTokens(code: code)
        }
    }
}
