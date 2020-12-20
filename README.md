Super basic rust bindings to JUCE6 juce_audio_devices module.

Right now it's meant for apps that need long-term access to devices and will close all devices at the same time, usually
app termination.

It also has no API to enumerate devices, so you need to know their names upfront.

Windows/ASIO support TBD.