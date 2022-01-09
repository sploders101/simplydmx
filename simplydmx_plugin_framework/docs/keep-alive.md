# Keep-Alive

The keep-alive queue allows a plugin to register tasks that have a definitive end. When the application
exits, this queue will be awaited before closing. For example, this queue can be used to quickly save a
copy of the working environment before exiting to avoid losing user data.
