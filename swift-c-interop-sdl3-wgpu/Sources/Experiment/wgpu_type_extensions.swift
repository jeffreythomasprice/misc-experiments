import CWGPU
import Foundation

extension String {
	public static func fromWGPUStringView(other: WGPUStringView) -> String {
		let buflen = other.length + 1
		let buf = UnsafeMutablePointer<CChar>.allocate(capacity: buflen)
		defer {
			buf.deallocate()
		}
		if other.length > 0 {
			memcpy(buf, other.data, other.length)
		}
		buf[other.length] = 0
		return String(cString: buf)
	}

	public func toWGPUStringView() -> WGPUStringView {
		let utf8 = self.utf8
		let count = utf8.count
		let buf = UnsafeMutablePointer<CChar>.allocate(capacity: count)
		var index = 0
		for byte in utf8 {
			buf[index] = CChar(byte)
			index += 1
		}
		return WGPUStringView(data: buf, length: count)
	}
}

extension Array {
	func asData<ResultType>(_ body: (Data?) -> ResultType) -> ResultType {
		self.withUnsafeBytes {
			let data: Data? =
				if let ptr = $0.baseAddress {
					Data(
						bytesNoCopy: UnsafeMutableRawPointer(mutating: ptr), count: $0.count,
						deallocator: Data.Deallocator.none)
				} else {
					nil
				}
			return body(data)
		}
	}
}
