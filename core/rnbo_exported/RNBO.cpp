#include "RNBO.h"

namespace RNBO {

    CoreObject::CoreObject() {}
    CoreObject::~CoreObject() {}

    void CoreObject::prepareToProcess(double, int) {}

    ParameterIndex CoreObject::getParameterIndexForID(const char*) const {
        return 0; // Mock implementation
    }

    void CoreObject::setParameterValue(ParameterIndex, float) {}

    void CoreObject::process(const float* const* inputs, int numInputs, float* const* outputs, int numOutputs, int numSamples) {
        // Just copy input to output or zero out
        if (numInputs > 0 && numOutputs > 0 && inputs[0] && outputs[0]) {
            for (int i = 0; i < numSamples; ++i) {
                outputs[0][i] = inputs[0][i];
            }
        } else if (numOutputs > 0 && outputs[0]) {
             for (int i = 0; i < numSamples; ++i) {
                outputs[0][i] = 0.0f;
             }
        }
    }
}