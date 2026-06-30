import UIKit

final class SettingsViewController: UIViewController {
    var onSettingsChanged: (() -> Void)?

    private let settings: AppSettings
    private let frameRateControl = UISegmentedControl(items: ["60 FPS", "120 FPS"])

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
        view.backgroundColor = UIColor(red: 0.035, green: 0.045, blue: 0.04, alpha: 1.0)
        installSettings()
    }

    private func installSettings() {
        let titleLabel = UILabel()
        titleLabel.text = "Settings"
        titleLabel.font = .systemFont(ofSize: 34, weight: .bold)
        titleLabel.textColor = UIColor(red: 0.88, green: 0.93, blue: 0.84, alpha: 1.0)

        let frameRateLabel = UILabel()
        frameRateLabel.text = "Frame Rate"
        frameRateLabel.font = .systemFont(ofSize: 17, weight: .semibold)
        frameRateLabel.textColor = UIColor(red: 0.80, green: 0.88, blue: 0.76, alpha: 1.0)

        frameRateControl.selectedSegmentIndex = AppSettings.supportedFrameRates
            .firstIndex(of: settings.targetFrameRate) ?? 1
        frameRateControl.selectedSegmentTintColor = UIColor(red: 0.20, green: 0.42, blue: 0.35, alpha: 1.0)
        frameRateControl.addTarget(self, action: #selector(frameRateChanged), for: .valueChanged)

        let doneButton = UIButton(type: .system)
        doneButton.setTitle("Done", for: .normal)
        doneButton.titleLabel?.font = .systemFont(ofSize: 17, weight: .semibold)
        doneButton.addTarget(self, action: #selector(closeSettings), for: .touchUpInside)

        let stack = UIStackView(arrangedSubviews: [titleLabel, frameRateLabel, frameRateControl, doneButton])
        stack.axis = .vertical
        stack.alignment = .fill
        stack.spacing = 18
        stack.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(stack)

        NSLayoutConstraint.activate([
            stack.leadingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.leadingAnchor, constant: 24),
            stack.trailingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.trailingAnchor, constant: -24),
            stack.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor, constant: 56),
            frameRateControl.heightAnchor.constraint(equalToConstant: 44),
            doneButton.heightAnchor.constraint(equalToConstant: 48),
        ])
    }

    @objc private func frameRateChanged() {
        let index = frameRateControl.selectedSegmentIndex
        guard AppSettings.supportedFrameRates.indices.contains(index) else {
            return
        }

        settings.targetFrameRate = AppSettings.supportedFrameRates[index]
        onSettingsChanged?()
    }

    @objc private func closeSettings() {
        if let navigationController {
            navigationController.popViewController(animated: true)
        } else {
            dismiss(animated: true)
        }
    }
}
