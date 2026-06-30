import UIKit

final class MainMenuViewController: UIViewController {
    private let settings: AppSettings

    init(settings: AppSettings = .shared) {
        self.settings = settings
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override var supportedInterfaceOrientations: UIInterfaceOrientationMask {
        .portrait
    }

    override var preferredInterfaceOrientationForPresentation: UIInterfaceOrientation {
        .portrait
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        navigationController?.setNavigationBarHidden(true, animated: false)
        view.backgroundColor = UIColor(red: 0.035, green: 0.045, blue: 0.04, alpha: 1.0)
        installMenu()
    }

    private func installMenu() {
        let titleLabel = UILabel()
        titleLabel.text = "Ashen Frontier"
        titleLabel.font = .systemFont(ofSize: 42, weight: .bold)
        titleLabel.textColor = UIColor(red: 0.88, green: 0.93, blue: 0.84, alpha: 1.0)
        titleLabel.textAlignment = .center

        let subtitleLabel = UILabel()
        subtitleLabel.text = "Native RTS Prototype"
        subtitleLabel.font = .systemFont(ofSize: 15, weight: .medium)
        subtitleLabel.textColor = UIColor(red: 0.55, green: 0.66, blue: 0.58, alpha: 1.0)
        subtitleLabel.textAlignment = .center

        let startButton = makeMenuButton(title: "Start")
        startButton.addTarget(self, action: #selector(startGame), for: .touchUpInside)

        let settingsButton = makeMenuButton(title: "Settings")
        settingsButton.addTarget(self, action: #selector(openSettings), for: .touchUpInside)

        let buttonStack = UIStackView(arrangedSubviews: [startButton, settingsButton])
        buttonStack.axis = .vertical
        buttonStack.spacing = 14
        buttonStack.translatesAutoresizingMaskIntoConstraints = false

        let stack = UIStackView(arrangedSubviews: [titleLabel, subtitleLabel, buttonStack])
        stack.axis = .vertical
        stack.alignment = .fill
        stack.spacing = 18
        stack.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(stack)

        NSLayoutConstraint.activate([
            stack.leadingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.leadingAnchor, constant: 28),
            stack.trailingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.trailingAnchor, constant: -28),
            stack.centerYAnchor.constraint(equalTo: view.centerYAnchor),
            startButton.heightAnchor.constraint(equalToConstant: 54),
            settingsButton.heightAnchor.constraint(equalToConstant: 54),
        ])
    }

    private func makeMenuButton(title: String) -> UIButton {
        var configuration = UIButton.Configuration.filled()
        configuration.title = title
        configuration.cornerStyle = .medium
        configuration.baseBackgroundColor = UIColor(red: 0.14, green: 0.34, blue: 0.29, alpha: 1.0)
        configuration.baseForegroundColor = UIColor(red: 0.91, green: 0.96, blue: 0.88, alpha: 1.0)

        let button = UIButton(configuration: configuration)
        button.titleLabel?.font = .systemFont(ofSize: 18, weight: .semibold)
        return button
    }

    @objc private func startGame() {
        let gameViewController = GameViewController(settings: settings)
        navigationController?.pushViewController(gameViewController, animated: true)
    }

    @objc private func openSettings() {
        let settingsViewController = SettingsViewController(settings: settings)
        navigationController?.pushViewController(settingsViewController, animated: true)
    }
}
