#include "RNBO.h"
#include <map>
#include <string>

extern "C" {
    // RNBO インスタンスの生成
    void* rnbo_create(double sample_rate, int block_size) {
        auto* obj = new RNBO::CoreObject();
        obj->prepareToProcess(sample_rate, block_size);
        return reinterpret_cast<void*>(obj);
    }

    // パラメータ名からインデックスへの変換（名前解決）
    int rnbo_get_param_index(void* ptr, const char* name) {
        auto* obj = reinterpret_cast<RNBO::CoreObject*>(ptr);
        RNBO::ParameterIndex idx = obj->getParameterIndexForID(name);
        if (idx == RNBO::INVALID_PARAMETER_INDEX) return -1;
        return static_cast<int>(idx);
    }

    // インデックス指定によるパラメータ更新
    void rnbo_set_parameter(void* ptr, int param_index, float value) {
        auto* obj = reinterpret_cast<RNBO::CoreObject*>(ptr);
        if (param_index >= 0) {
            obj->setParameterValue(param_index, value);
        }
    }

    // 全二重（同時入出力）プロセッシングループ
    void rnbo_process(void* ptr, const float* input, float* output, int num_samples) {
        auto* obj = reinterpret_cast<RNBO::CoreObject*>(ptr);

        // モノラル入出力を想定（ステレオの場合は適宜拡張）
        const float* inputs[] = { input };
        float* outputs[] = { output };

        obj->process(inputs, 1, outputs, 1, num_samples);
    }

    // インスタンスの破棄
    void rnbo_destroy(void* ptr) {
        delete reinterpret_cast<RNBO::CoreObject*>(ptr);
    }
}