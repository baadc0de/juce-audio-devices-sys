#include <juce_audio_devices/juce_audio_devices.h>
#include <iostream>
#include <memory>
#include <vector>

using namespace std;
using namespace juce;

extern "C" {
size_t __unused get_devices() {
    AudioDeviceManager mgr;
    size_t i = 0;
    for (auto dev_type : mgr.getAvailableDeviceTypes()) {
        std::cout << dev_type->getTypeName() << std::endl;
        for (auto dev : dev_type->getDeviceNames()) {
            std::cout << dev_type->getTypeName() << ": " << dev.toStdString() << std::endl;
            i++;
        }
    }
    return i;
}

enum {
    ERR_NO_DRIVER = -1,
    ERR_NO_DEVICE = -2,
    ERR_MGR_CONSTRUCT = -3,
    ERR_DEV_CONSTRUCT = -4,
};

typedef void (*DevIOCallback)(int ctx, const float **inputs, int num_inputs, float **outputs, int num_outputs,
                              int num_samples);

struct AudioDevice : public AudioIODeviceCallback {
    unique_ptr<AudioIODevice> device;
    DevIOCallback callback;
    int ctx;

    void audioDeviceIOCallback(const float **inputChannelData,
                               int numInputChannels,
                               float **outputChannelData,
                               int numOutputChannels,
                               int numSamples) override {
        this->callback(this->ctx, inputChannelData, numInputChannels, outputChannelData, numOutputChannels, numSamples);
    }

    AudioDevice(AudioIODevice *_device, int _ctx, DevIOCallback _callback, int input_channels,
                int output_channels, double sample_rate, int buffer_size) : device(_device), ctx(_ctx),
                                                                            callback(_callback) {
        device->setAudioPreprocessingEnabled(false);
        BigInteger inputs;
        BigInteger outputs;

        inputs.setRange(0, input_channels, true);
        outputs.setRange(0, output_channels, true);

        device->open(inputs, outputs, sample_rate, buffer_size);
        device->start(this);
    }

    void audioDeviceAboutToStart(AudioIODevice *) override {
    }

    void audioDeviceStopped() override {
    }

    void audioDeviceError(const String &errorMessage) override {
        cerr << "audio device error" << errorMessage.toStdString() << endl;
    }
};

static vector<AudioDevice> devices;
static unique_ptr<AudioDeviceManager> mgr;

int __unused activate_device(const char *driver, const char *output_name, const char *input_name, int input_channels,
                             int output_channels, double sample_rate, int buffer_size, int ctx,
                             DevIOCallback callback) {
    try {
        if (!mgr) {
            mgr.reset(new AudioDeviceManager());
        }
    } catch (...) {
        return ERR_MGR_CONSTRUCT;
    }

    for (auto dev_type : mgr->getAvailableDeviceTypes()) {
        if (dev_type->getTypeName() == driver) {
            dev_type->scanForDevices();
            try {
                auto *dev_raw = dev_type->createDevice(output_name, input_name);
                if (!dev_raw) {
                    return ERR_NO_DEVICE;
                }

                devices.emplace_back(dev_raw, ctx, callback, input_channels, output_channels, sample_rate, buffer_size);
                return ctx;
            } catch (...) {
                return ERR_DEV_CONSTRUCT;
            }
        }
    }
    return ERR_NO_DRIVER;
}

void stop_devices() {
    devices.clear();
    mgr.reset();
}
}