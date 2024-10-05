```
sudo apt-get install inotify-tools
```

```
watchexec -r --wrap-process=session gleam run
watchexec -r --wrap-process=session gleam test
```