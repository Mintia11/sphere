use ash::vk;

pub trait VideoCodecExtension {
    /// Device extensions this codec needs enabled (e.g. VK_KHR_video_decode_h264)
    fn device_extensions(&self) -> &'static [&'static std::ffi::CStr];

    /// The video codec operation this contributes, for capability/profile queries
    fn codec_operation(&self) -> vk::VideoCodecOperationFlagsKHR;
}
