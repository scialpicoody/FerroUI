#!/usr/bin/env swift
// Copyright (c) FerroUI contributors. Licensed under the MIT License.
// Capture native AppKit reference metrics on macOS 26. Build with the macOS 26 SDK:
//   swift scripts/macos-theme/capture-native.swift --output /tmp/ferroui-macos26
// Add --screen-capture for WindowServer images (requires Screen Recording permission).
import AppKit
import Foundation

struct Options {
    var output = URL(fileURLWithPath: "/tmp/ferroui-macos26", isDirectory: true)
    var screenCapture = false

    init(arguments: [String]) throws {
        var index = 0
        while index < arguments.count {
            switch arguments[index] {
            case "--output":
                index += 1
                guard index < arguments.count else { throw ProbeError.message("--output requires a directory") }
                output = URL(fileURLWithPath: arguments[index], isDirectory: true)
            case "--screen-capture": screenCapture = true
            case "--help":
                print("capture-native.swift [--output DIRECTORY] [--screen-capture]")
                exit(0)
            default: throw ProbeError.message("Unknown option: \(arguments[index])")
            }
            index += 1
        }
    }
}

enum ProbeError: Error, CustomStringConvertible {
    case message(String)
    var description: String {
        switch self { case let .message(value): return value }
    }
}

struct RGBA: Codable {
    let red: CGFloat
    let green: CGFloat
    let blue: CGFloat
    let alpha: CGFloat

    init(_ color: NSColor) throws {
        guard let rgb = color.usingColorSpace(.sRGB) else {
            throw ProbeError.message("Cannot convert a system color to sRGB")
        }
        red = rgb.redComponent; green = rgb.greenComponent
        blue = rgb.blueComponent; alpha = rgb.alphaComponent
    }
}

struct ControlMetric: Codable {
    let identifier: String
    let size: String
    let className: String
    let enabled: Bool
    let width: CGFloat
    let height: CGFloat
    let intrinsicWidth: CGFloat
    let intrinsicHeight: CGFloat
    let fittingWidth: CGFloat
    let fittingHeight: CGFloat
    let fontName: String?
    let fontSize: CGFloat?
    let ascender: CGFloat?
    let descender: CGFloat?
    let leading: CGFloat?
    let alignmentInsets: [CGFloat]
}

struct Capture: Codable {
    let appearance: String
    let activeWindow: Bool
    let scale: CGFloat
    let viewImage: String
    let windowServerImage: String?
    let colors: [String: RGBA]
    let controls: [ControlMetric]
}

struct Manifest: Codable {
    let schemaVersion: Int
    let osVersion: String
    let systemVersion: String
    let reduceMotion: Bool
    let reduceTransparency: Bool
    let increaseContrast: Bool
    let captureNotes: [String]
    let captures: [Capture]
}

@MainActor
final class ReferenceFixture {
    let window: NSWindow
    private(set) var controls: [(String, String, NSControl)] = []

    @available(macOS 26.0, *)
    init(appearance: NSAppearance) {
        window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 1180, height: 980),
                          styleMask: [.titled, .closable, .miniaturizable, .resizable],
                          backing: .buffered, defer: false)
        window.title = "FerroUI — macOS 26 native reference"
        window.appearance = appearance
        window.isReleasedWhenClosed = false
        for (name, type) in [("window-close", NSWindow.ButtonType.closeButton),
                             ("window-minimize", .miniaturizeButton), ("window-zoom", .zoomButton)] {
            if let button = window.standardWindowButton(type) {
                controls.append((name, "window-chrome", button))
            }
        }

        let content = NSStackView()
        content.orientation = .vertical
        content.alignment = .leading
        content.spacing = 16
        content.edgeInsets = NSEdgeInsets(top: 20, left: 20, bottom: 20, right: 20)
        content.translatesAutoresizingMaskIntoConstraints = false
        let host = NSView()
        window.contentView = host
        host.addSubview(content)
        NSLayoutConstraint.activate([
            content.leadingAnchor.constraint(equalTo: host.leadingAnchor),
            content.trailingAnchor.constraint(equalTo: host.trailingAnchor),
            content.topAnchor.constraint(equalTo: host.topAnchor),
            content.bottomAnchor.constraint(lessThanOrEqualTo: host.bottomAnchor)
        ])

        let heading = NSTextField(labelWithString: "Native AppKit control metrics")
        heading.font = .boldSystemFont(ofSize: 20)
        content.addArrangedSubview(heading)
        let description = NSTextField(labelWithString:
            "Reference only • macOS 26 • actual system accent and accessibility settings • sizes in points")
        description.textColor = .secondaryLabelColor
        content.addArrangedSubview(description)

        let sizes: [(String, NSControl.ControlSize)] = [
            ("mini", .mini), ("small", .small), ("regular", .regular),
            ("large", .large), ("extra-large", .extraLarge)
        ]
        for (name, size) in sizes {
            let title = NSTextField(labelWithString: name)
            title.font = .boldSystemFont(ofSize: 13)
            content.addArrangedSubview(title)
            let first = row()
            let second = row()

            let push = NSButton(title: "Button", target: nil, action: nil)
            push.bezelStyle = .rounded
            add(push, id: "push", size: name, controlSize: size, to: first)
            let primary = NSButton(title: "Default", target: nil, action: nil)
            primary.bezelStyle = .rounded
            primary.keyEquivalent = "\r"
            add(primary, id: "default-push", size: name, controlSize: size, to: first)
            let disabled = NSButton(title: "Disabled", target: nil, action: nil)
            disabled.bezelStyle = .rounded
            disabled.isEnabled = false
            add(disabled, id: "disabled-push", size: name, controlSize: size, to: first)
            let check = NSButton(checkboxWithTitle: "Checkbox", target: nil, action: nil)
            check.state = .on
            add(check, id: "checkbox-on", size: name, controlSize: size, to: first)
            let mixed = NSButton(checkboxWithTitle: "Mixed", target: nil, action: nil)
            mixed.allowsMixedState = true
            mixed.state = .mixed
            add(mixed, id: "checkbox-mixed", size: name, controlSize: size, to: first)
            let radio = NSButton(radioButtonWithTitle: "Radio", target: nil, action: nil)
            radio.state = .on
            add(radio, id: "radio-on", size: name, controlSize: size, to: first)
            let toggle = NSSwitch()
            toggle.state = .on
            add(toggle, id: "switch-on", size: name, controlSize: size, to: first)

            let field = NSTextField(string: "Text field")
            add(field, id: "text-field", size: name, controlSize: size, to: second, width: 140)
            let search = NSSearchField(string: "")
            search.placeholderString = "Search"
            add(search, id: "search-field", size: name, controlSize: size, to: second, width: 140)
            let popup = NSPopUpButton(frame: .zero, pullsDown: false)
            popup.addItems(withTitles: ["First option", "Second option", "Third option"])
            add(popup, id: "popup", size: name, controlSize: size, to: second)
            let segments = NSSegmentedControl(labels: ["Day", "Week", "Month"],
                                               trackingMode: .selectOne, target: nil, action: nil)
            segments.selectedSegment = 1
            add(segments, id: "segmented", size: name, controlSize: size, to: second)
            let slider = NSSlider(value: 65, minValue: 0, maxValue: 100, target: nil, action: nil)
            add(slider, id: "slider", size: name, controlSize: size, to: second, width: 140)
            let stepper = NSStepper()
            stepper.minValue = 0
            stepper.maxValue = 100
            stepper.doubleValue = 25
            add(stepper, id: "stepper", size: name, controlSize: size, to: second)
            content.addArrangedSubview(first)
            content.addArrangedSubview(second)
        }

        let materialTitle = NSTextField(labelWithString: "Native glass surface — compare only with WindowServer capture")
        materialTitle.font = .boldSystemFont(ofSize: 13)
        content.addArrangedSubview(materialTitle)
        let glass = NSGlassEffectView()
        glass.cornerRadius = 12
        let glassContent = NSStackView()
        glassContent.edgeInsets = NSEdgeInsets(top: 14, left: 16, bottom: 14, right: 16)
        glassContent.addArrangedSubview(NSTextField(labelWithString: "NSGlassEffectView"))
        glass.contentView = glassContent
        content.addArrangedSubview(glass)
        window.center()
    }

    private func row() -> NSStackView {
        let result = NSStackView()
        result.orientation = .horizontal
        result.alignment = .centerY
        result.spacing = 16
        return result
    }

    private func add(_ control: NSControl, id: String, size: String, controlSize: NSControl.ControlSize,
                     to row: NSStackView, width: CGFloat? = nil) {
        control.controlSize = controlSize
        control.font = .systemFont(ofSize: NSFont.systemFontSize(for: controlSize))
        control.sizeToFit()
        row.addArrangedSubview(control)
        if let width { control.widthAnchor.constraint(equalToConstant: width).isActive = true }
        control.setContentCompressionResistancePriority(.required, for: .vertical)
        controls.append((id, size, control))
    }

    func metrics() -> [ControlMetric] {
        controls.map { id, size, control in
            let intrinsic = control.intrinsicContentSize
            let fitting = control.fittingSize
            let font = control.font
            let insets = control.alignmentRectInsets
            return ControlMetric(identifier: id, size: size, className: NSStringFromClass(type(of: control)),
                enabled: control.isEnabled, width: control.frame.width, height: control.frame.height,
                intrinsicWidth: intrinsic.width, intrinsicHeight: intrinsic.height,
                fittingWidth: fitting.width, fittingHeight: fitting.height,
                fontName: font?.fontName, fontSize: font?.pointSize, ascender: font?.ascender,
                descender: font?.descender, leading: font?.leading,
                alignmentInsets: [insets.top, insets.left, insets.bottom, insets.right])
        }
    }
}

@MainActor
func settle() {
    RunLoop.current.run(until: Date(timeIntervalSinceNow: 0.2))
}

@MainActor
func captureView(_ window: NSWindow, to url: URL) throws {
    guard let view = window.contentView?.superview else { throw ProbeError.message("Window has no frame view") }
    view.layoutSubtreeIfNeeded()
    guard let bitmap = view.bitmapImageRepForCachingDisplay(in: view.bounds) else {
        throw ProbeError.message("Cannot allocate view capture bitmap")
    }
    view.cacheDisplay(in: view.bounds, to: bitmap)
    guard let png = bitmap.representation(using: .png, properties: [:]) else {
        throw ProbeError.message("Cannot encode view capture")
    }
    try png.write(to: url, options: .atomic)
}

func captureWindow(_ number: Int, to url: URL) throws {
    let task = Process()
    task.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
    task.arguments = ["-x", "-o", "-l", String(number), url.path]
    try task.run()
    task.waitUntilExit()
    guard task.terminationStatus == 0, FileManager.default.fileExists(atPath: url.path) else {
        throw ProbeError.message("WindowServer capture failed. Grant Screen Recording permission or omit --screen-capture.")
    }
}

@MainActor
func colors(appearance: NSAppearance) throws -> [String: RGBA] {
    let semantic: [(String, NSColor)] = [
        ("controlAccent", .controlAccentColor), ("windowBackground", .windowBackgroundColor),
        ("controlBackground", .controlBackgroundColor), ("control", .controlColor),
        ("controlText", .controlTextColor), ("disabledControlText", .disabledControlTextColor),
        ("label", .labelColor), ("secondaryLabel", .secondaryLabelColor),
        ("tertiaryLabel", .tertiaryLabelColor), ("quaternaryLabel", .quaternaryLabelColor),
        ("keyboardFocusIndicator", .keyboardFocusIndicatorColor), ("separator", .separatorColor),
        ("selectedContentBackground", .selectedContentBackgroundColor),
        ("unemphasizedSelectedContentBackground", .unemphasizedSelectedContentBackgroundColor),
        ("selectedTextBackground", .selectedTextBackgroundColor), ("selectedText", .selectedTextColor),
        ("text", .textColor), ("textBackground", .textBackgroundColor),
        ("placeholderText", .placeholderTextColor), ("link", .linkColor)
    ]
    var result: [String: RGBA] = [:]
    var failure: Error?
    appearance.performAsCurrentDrawingAppearance {
        do { for (name, color) in semantic { result[name] = try RGBA(color) } }
        catch { failure = error }
    }
    if let failure { throw failure }
    return result
}

@MainActor
func run() throws {
    let options = try Options(arguments: Array(CommandLine.arguments.dropFirst()))
    let version = ProcessInfo.processInfo.operatingSystemVersion
    guard version.majorVersion == 26 else {
        throw ProbeError.message("This fixture targets macOS 26 exactly; refusing to label macOS \(version.majorVersion) output as macOS 26.")
    }
    guard #available(macOS 26.0, *) else { return }
    try FileManager.default.createDirectory(at: options.output, withIntermediateDirectories: true)
    let app = NSApplication.shared
    app.setActivationPolicy(.accessory)
    app.finishLaunching()
    app.activate(ignoringOtherApps: true)
    var captures: [Capture] = []
    let workspace = NSWorkspace.shared
    let highContrast = workspace.accessibilityDisplayShouldIncreaseContrast
    let appearances: [(String, NSAppearance.Name)] = [
        ("light", highContrast ? .accessibilityHighContrastAqua : .aqua),
        ("dark", highContrast ? .accessibilityHighContrastDarkAqua : .darkAqua)
    ]

    for (name, appearanceName) in appearances {
        guard let appearance = NSAppearance(named: appearanceName) else {
            throw ProbeError.message("Cannot resolve \(name) appearance")
        }
        let fixture = ReferenceFixture(appearance: appearance)
        fixture.window.makeKeyAndOrderFront(nil)
        settle()
        let focusWindow = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 160, height: 80),
                                   styleMask: [.titled], backing: .buffered, defer: false)
        focusWindow.isReleasedWhenClosed = false
        focusWindow.title = "Inactive reference"
        for active in [true, false] {
            if active { fixture.window.makeKeyAndOrderFront(nil) }
            else { focusWindow.makeKeyAndOrderFront(nil) }
            settle()
            let stem = name + (active ? "-active" : "-inactive")
            let viewName = stem + "-view.png"
            try captureView(fixture.window, to: options.output.appendingPathComponent(viewName))
            var serverName: String?
            if options.screenCapture {
                let filename = stem + "-window.png"
                try captureWindow(fixture.window.windowNumber,
                                  to: options.output.appendingPathComponent(filename))
                serverName = filename
            }
            captures.append(Capture(appearance: name, activeWindow: fixture.window.isKeyWindow,
                scale: fixture.window.backingScaleFactor, viewImage: viewName,
                windowServerImage: serverName, colors: try colors(appearance: appearance), controls: fixture.metrics()))
        }
        focusWindow.close()
        fixture.window.close()
    }
    let manifest = Manifest(schemaVersion: 1,
        osVersion: "\(version.majorVersion).\(version.minorVersion).\(version.patchVersion)",
        systemVersion: ProcessInfo.processInfo.operatingSystemVersionString,
        reduceMotion: workspace.accessibilityDisplayShouldReduceMotion,
        reduceTransparency: workspace.accessibilityDisplayShouldReduceTransparency,
        increaseContrast: workspace.accessibilityDisplayShouldIncreaseContrast,
        captureNotes: [
            "Measurements are native AppKit points and alignment/fitting metrics, not normative Apple specifications.",
            "View snapshots may omit compositor effects. Use optional WindowServer captures for visual comparisons.",
            "The fixture does not capture hover/pressed sequences, open menus, sheets, or animation curves.",
            "Run again under each required system accent, accessibility preference, and display scaling configuration.",
            "macOS 27 changes are intentionally excluded by the OS-major-version guard."
        ], captures: captures)
    let encoder = JSONEncoder()
    encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
    let manifestURL = options.output.appendingPathComponent("native-reference.json")
    try encoder.encode(manifest).write(to: manifestURL, options: .atomic)
    print(manifestURL.path)
}

do {
    try MainActor.assumeIsolated { try run() }
} catch {
    fputs("macOS 26 capture: \(error)\n", stderr)
    exit(1)
}
