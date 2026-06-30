import UIKit

@main
final class AppDelegate: UIResponder, UIApplicationDelegate {
    var window: UIWindow?

    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {
        let window = UIWindow(frame: UIScreen.main.bounds)
        let menuViewController = MainMenuViewController(settings: .shared)
        window.rootViewController = UINavigationController(rootViewController: menuViewController)
        window.makeKeyAndVisible()
        self.window = window
        return true
    }
}
