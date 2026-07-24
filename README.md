# PV Unlocker

This is a program used to check the version of an MER PanelView runtime or FactoryTalk View PanelView archive file (APA). If the MER is locked or marked as Never allow conversion back into a Project file. This can strip that, and allow the project to be recovered.

My initial crude version of this I had set up to right click on an MER or APA and get the version and unlock it. I manually edited the registry on my machine for this to work. Right now as I haven't created an installer, unless you manually set this up in the registry you won't be able to right click an MER or APA and run this.

### Limitations
While the technique is slightly different (internally), using the strip_protection function can allow a V4 MER to be re-imported as a project file. There may however be some limitations here. I have minimal v4 files to test with (and none I can add into the test suite). One limitation I have noticed is that while the alarm triggers and states came through the alarm messages are all "?", so it appears the alarm messages are not directly recoverable.

I have been handed a version 3.20 MER file, it appears the technique to recover a v4 file does not work on it. I will see what can be done, but as of now MER files older than V4 are not recoverable.

### To Use
Run the executable, click 'Select Files', and pick the files you want to open. A list of files will appear with the file name, version number, and an icon representing the locked/unlocked status. If you want to unlock a file, click the red locked icon next to it to unlock it (this will directly modify the file, so if you are worried about it breaking, work on a copy).

You can also run the file from the command line and pass it a list of files such as: `pv-unlocker.exe *.mer` and it will show you the status of those files.


## Future Goals
I had thought about instead of a small app, making this a windows shell extension, overlay a lock icon if the file is locked, allow you to right click and unlock it, Maybe see the Runtime version in the File Details pane. I have never made a windows shell extension before, and haven't had the time to dig into it.
