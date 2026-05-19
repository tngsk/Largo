#include "RNBO.h"
#include <cmath>
#include <cstring>

namespace RNBO {

    class CoreObjectImpl {
    public:
        double phase = 0.0;
        double sampleRate = 44100.0;
        float frequency = 440.0f;
        float amplitude = 0.0f;
    };

    CoreObject::CoreObject() {
        _impl = new CoreObjectImpl();
    }

    CoreObject::~CoreObject() {
        delete reinterpret_cast<CoreObjectImpl*>(_impl);
    }

    void CoreObject::prepareToProcess(double sampleRate, int /*maxBlockSize*/) {
        auto* impl = reinterpret_cast<CoreObjectImpl*>(_impl);
        impl->sampleRate = sampleRate;
    }

    ParameterIndex CoreObject::getParameterIndexForID(const char* paramId) const {
        if (std::strcmp(paramId, "frequency") == 0) return 0;
        if (std::strcmp(paramId, "amplitude") == 0) return 1;
        return INVALID_PARAMETER_INDEX;
    }

    void CoreObject::setParameterValue(ParameterIndex index, float value) {
        auto* impl = reinterpret_cast<CoreObjectImpl*>(_impl);
        if (index == 0) impl->frequency = value;
        else if (index == 1) impl->amplitude = value;
    }

    void CoreObject::process(const float* const* /*inputs*/, int /*numInputs*/, float* const* outputs, int numOutputs, int numSamples) {
        auto* impl = reinterpret_cast<CoreObjectImpl*>(_impl);
        if (numOutputs > 0 && outputs[0]) {
            float* out = outputs[0];
            double phase_inc = 2.0 * M_PI * impl->frequency / impl->sampleRate;
            
            for (int i = 0; i < numSamples; ++i) {
                out[i] = (float)(std::sin(impl->phase) * impl->amplitude);
                impl->phase += phase_inc;
                if (impl->phase >= 2.0 * M_PI) impl->phase -= 2.0 * M_PI;
            }
        }
    }
}