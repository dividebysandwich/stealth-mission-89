# The radio voices (src/voice_dialog.rs) and the coop radio's notices (src/voice).

vd-title = VOICES
vd-back = Back to the options
vd-radio-voices = Radio voices
vd-none-yet = No voice yet: download one below. Until then the calls are text.
vd-pilot = PILOT
vd-controllers = CONTROLLERS
vd-none = None (text only)
vd-speaker = speaker
    .one = One of the voice's speakers (drag, or click and type)
    .corpus = { $corpus } in the recordings
    .preview = Preview to hear it
vd-preview = PREVIEW
    .facts = Hear it on the radio
vd-vary = A different controller at each station
    .facts =
        Tower, approach, departure and GCA each talk with another of the voice's speakers
        The tower's is the one chosen above
vd-installed = Installed
vd-licence-unknown = Licence unknown: see the voice's MODEL_CARD
vd-speakers = Speakers: { $count }
vd-beside-game = beside the game
    .facts = Found in a voices folder next to the game: remove it there
vd-keep = KEEP
vd-delete =
    .facts = Removes its files (asks again)
vd-deleted = { $name } deleted
vd-download = Download
vd-no-folder = No folder to keep voices in (no home directory)
vd-download-button = DOWNLOAD
    .facts =
        From huggingface.co/rhasspy/piper-voices
        Checked before it is used
vd-open = OPEN
    .facts =
        Opens the voices folder
        Piper voices (.onnx with its .onnx.json) put here are found too
vd-folder = Voices folder: { $path }
vd-downloading = Downloading { $name }
vd-installed-ok = { $name } installed
vd-cancelled = { $name }: download cancelled
vd-failed = { $name }: download failed: { $error }
vr-pilot = Pilot { $id }
vr-on-radio = { $name } on the radio
vr-transmitting = Transmitting
vr-microphone = Microphone: { $error }
