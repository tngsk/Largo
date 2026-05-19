#ifndef RNBO_H
#define RNBO_H

#include <cstdint>
#include <string>

namespace RNBO {
    typedef int ParameterIndex;
    const ParameterIndex INVALID_PARAMETER_INDEX = -1;

    class CoreObject {
    public:
        CoreObject();
        ~CoreObject();
        void prepareToProcess(double sampleRate, int maxBlockSize);
        ParameterIndex getParameterIndexForID(const char* paramId) const;
        void setParameterValue(ParameterIndex index, float value);
        void process(const float* const* inputs, int numInputs, float* const* outputs, int numOutputs, int numSamples);
    private:
        void* _impl;
    };
}

#endif // RNBO_H