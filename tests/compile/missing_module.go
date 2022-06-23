package main

import "thismoduledoesnotexist"

func main() {
    fmt.Println("hello world")
}

// err=LinkerError(Loader(ModuleNotFound("thismoduledoesnotexist")))
