# checked_wipe
Usage of this tool is rather simple. 

```
sudo cargo run --release
```

The program will then walk you through the process as it zeroes out your drive!

If you want to specify the number of iterations of zeroing you would like, you can do so as follows:

```
sudo cargo run --release -- -n <loops>
```

If you are in need of wiping your device quickly (or slightly less slowly as the case may be), there is an option to disable the self-check that occurs at the end of the looping process.

``` 
sudo cargo run --release -- -c
```

More information can be found using --help :)
