// ffi_example.cpp
// C側から呼び出されるC++実装のテンプレート

#include <iostream>

// C++側の実際のクラス
class AudioProcessor {
public:
    AudioProcessor() {
        std::cout << "[C++] AudioProcessor Created" << std::endl;
    }
    ~AudioProcessor() {
        std::cout << "[C++] AudioProcessor Destroyed" << std::endl;
    }

    float process(float input) {
        return input * 2.0f;
    }
};

// C++からCインターフェースを公開
extern "C" {

    void* create_processor() {
        return new AudioProcessor();
    }

    float process_audio(void* ptr, float value) {
        if (ptr == nullptr) return 0.0f;
        AudioProcessor* processor = static_cast<AudioProcessor*>(ptr);
        return processor->process(value);
    }

    void destroy_processor(void* ptr) {
        if (ptr != nullptr) {
            AudioProcessor* processor = static_cast<AudioProcessor*>(ptr);
            delete processor;
        }
    }

}
