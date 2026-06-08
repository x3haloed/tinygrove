import AppKit
import AVFoundation
import CoreMedia
import Foundation
import ScreenCaptureKit

struct RecorderArgs {
  let outputURL: URL
  let durationSeconds: Double
  let processID: pid_t?
  let bundleID: String
  let windowOwnerFilter: String
  let windowTitleFilter: String
  let framerate: Double
  let maxWaitSeconds: Double
}

final class RecorderDelegate: NSObject, SCRecordingOutputDelegate, SCStreamDelegate {
  private let startSemaphore = DispatchSemaphore(value: 0)
  private let finishSemaphore = DispatchSemaphore(value: 0)
  private var startError: Error?
  private var finishError: Error?

  func recordingOutputDidStartRecording(_ recordingOutput: SCRecordingOutput) {
    startSemaphore.signal()
  }

  func recordingOutput(_ recordingOutput: SCRecordingOutput, didFailWithError error: Error) {
    startError = error
    startSemaphore.signal()
    finishError = error
    finishSemaphore.signal()
  }

  func recordingOutputDidFinishRecording(_ recordingOutput: SCRecordingOutput) {
    finishSemaphore.signal()
  }

  func stream(_ stream: SCStream, didStopWithError error: Error) {
    finishError = error
    finishSemaphore.signal()
  }

  func waitForStart(timeout: DispatchTime) throws {
    guard startSemaphore.wait(timeout: timeout) == .success else {
      throw NSError(domain: "RecorderDelegate", code: 1, userInfo: [NSLocalizedDescriptionKey: "Timed out waiting for recording to start"])
    }
    if let startError {
      throw startError
    }
  }

  func waitForFinish(timeout: DispatchTime) throws {
    guard finishSemaphore.wait(timeout: timeout) == .success else {
      throw NSError(domain: "RecorderDelegate", code: 2, userInfo: [NSLocalizedDescriptionKey: "Timed out waiting for recording to finish"])
    }
    if let finishError {
      throw finishError
    }
  }
}

@main
struct Main {
  static func main() async {
    do {
      _ = NSApplication.shared
      NSApp.setActivationPolicy(.accessory)
      let args = try parseArgs()
      try await record(args)
    } catch {
      fputs("ScreenCaptureKit recorder failed: \(error)\n", stderr)
      exit(1)
    }
  }

  private static func parseArgs() throws -> RecorderArgs {
    var outputPath: String?
    var duration = 8.0
    var processID: pid_t?
    var bundleID = "org.godotengine.godot"
    var windowOwnerFilter = "Godot"
    var windowTitleFilter = "Starship MMO"
    var framerate = 30.0
    var maxWaitSeconds = 20.0

    var index = 1
    let arguments = CommandLine.arguments
    while index < arguments.count {
      let arg = arguments[index]
      switch arg {
      case "--output":
        index += 1
        outputPath = index < arguments.count ? arguments[index] : nil
      case "--duration":
        index += 1
        if index < arguments.count, let value = Double(arguments[index]) {
          duration = value
        }
      case "--pid":
        index += 1
        if index < arguments.count, let value = Int32(arguments[index]) {
          processID = value
        }
      case "--bundle-id":
        index += 1
        if index < arguments.count {
          bundleID = arguments[index]
        }
      case "--owner":
        index += 1
        if index < arguments.count {
          windowOwnerFilter = arguments[index]
        }
      case "--title":
        index += 1
        if index < arguments.count {
          windowTitleFilter = arguments[index]
        }
      case "--fps":
        index += 1
        if index < arguments.count, let value = Double(arguments[index]) {
          framerate = value
        }
      case "--wait":
        index += 1
        if index < arguments.count, let value = Double(arguments[index]) {
          maxWaitSeconds = value
        }
      default:
        break
      }
      index += 1
    }

    guard let outputPath else {
      throw NSError(domain: "RecorderArgs", code: 1, userInfo: [NSLocalizedDescriptionKey: "Missing --output"])
    }

    return RecorderArgs(
      outputURL: URL(fileURLWithPath: outputPath),
      durationSeconds: duration,
      processID: processID,
      bundleID: bundleID,
      windowOwnerFilter: windowOwnerFilter,
      windowTitleFilter: windowTitleFilter,
      framerate: framerate,
      maxWaitSeconds: maxWaitSeconds
    )
  }

  private static func record(_ args: RecorderArgs) async throws {
    if FileManager.default.fileExists(atPath: args.outputURL.path) {
      try FileManager.default.removeItem(at: args.outputURL)
    }

    let window = try await waitForWindow(
      processID: args.processID,
      bundleID: args.bundleID,
      ownerFilter: args.windowOwnerFilter,
      titleFilter: args.windowTitleFilter,
      maxWaitSeconds: args.maxWaitSeconds
    )
    let contentFilter = SCContentFilter(desktopIndependentWindow: window)
    let contentInfo = SCShareableContent.info(for: contentFilter)
    let scale = max(1.0, Double(contentInfo.pointPixelScale))
    let contentRect = contentInfo.contentRect
    let targetWidth = max(1, Int((contentRect.width * scale).rounded(.up)))
    let targetHeight = max(1, Int((contentRect.height * scale).rounded(.up)))

    let configuration = SCStreamConfiguration()
    configuration.width = targetWidth
    configuration.height = targetHeight
    configuration.minimumFrameInterval = CMTime(value: 1, timescale: CMTimeScale(max(1, Int(args.framerate.rounded()))))
    configuration.pixelFormat = kCVPixelFormatType_32BGRA
    configuration.scalesToFit = true
    configuration.preservesAspectRatio = true
    configuration.showsCursor = false
    configuration.showMouseClicks = false
    configuration.queueDepth = 3
    configuration.shouldBeOpaque = true
    configuration.captureResolution = .best
    configuration.includeChildWindows = true
    configuration.capturesAudio = false

    let delegate = RecorderDelegate()
    let stream = SCStream(filter: contentFilter, configuration: configuration, delegate: delegate)

    let recordingConfiguration = SCRecordingOutputConfiguration()
    recordingConfiguration.outputURL = args.outputURL
    recordingConfiguration.videoCodecType = AVVideoCodecType.h264
    recordingConfiguration.outputFileType = AVFileType.mp4
    let recordingOutput = SCRecordingOutput(configuration: recordingConfiguration, delegate: delegate)

    try stream.addRecordingOutput(recordingOutput)

    try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
      stream.startCapture { error in
        if let error {
          continuation.resume(throwing: error)
        } else {
          continuation.resume(returning: ())
        }
      }
    }

    try delegate.waitForStart(timeout: .now() + .seconds(10))
    try await Task.sleep(for: .seconds(args.durationSeconds))

    try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
      stream.stopCapture { error in
        if let error {
          continuation.resume(throwing: error)
        } else {
          continuation.resume(returning: ())
        }
      }
    }

    try delegate.waitForFinish(timeout: .now() + .seconds(10))
  }

  private static func waitForWindow(processID: pid_t?, bundleID: String, ownerFilter: String, titleFilter: String, maxWaitSeconds: Double) async throws -> SCWindow {
    let deadline = Date().addingTimeInterval(maxWaitSeconds)
    let ownerLower = ownerFilter.lowercased()
    let titleLower = titleFilter.lowercased()
    let bundleLower = bundleID.lowercased()

    while Date() < deadline {
      let content = try await SCShareableContent.current
      if let window = selectWindow(from: content.windows, processID: processID, bundleID: bundleLower, ownerFilter: ownerLower, titleFilter: titleLower) {
        return window
      }

      try await Task.sleep(for: .milliseconds(250))
    }

    throw NSError(domain: "ScreenCaptureKit", code: 2, userInfo: [NSLocalizedDescriptionKey: "Timed out waiting for a Godot game window"])
  }

  private static func selectWindow(from windows: [SCWindow], processID: pid_t?, bundleID: String, ownerFilter: String, titleFilter: String) -> SCWindow? {
    let matches = windows.compactMap { window -> (SCWindow, Int, Int)? in
      guard let app = window.owningApplication else { return nil }

      if let processID, app.processID != processID {
        return nil
      }

      let owner = app.applicationName.lowercased()
      let appBundle = app.bundleIdentifier.lowercased()
      if !owner.isEmpty || !appBundle.isEmpty {
        let ownerMatches = owner.contains(ownerFilter) || owner.contains(bundleID)
        let bundleMatches = appBundle == bundleID || appBundle.contains(ownerFilter) || appBundle.contains(bundleID)
        if !ownerMatches && !bundleMatches {
          return nil
        }
      }

      if processID == nil {
        let title = (window.title ?? "").lowercased()
        guard title.contains(titleFilter), !title.contains("project manager") else {
          return nil
        }
      }

      let layerScore = window.windowLayer == 0 ? 0 : 1
      let area = Int(window.frame.width * window.frame.height)
      return (window, layerScore, -area)
    }

    return matches.sorted { lhs, rhs in
      if lhs.1 != rhs.1 { return lhs.1 < rhs.1 }
      return lhs.2 < rhs.2
    }.first?.0
  }
}
