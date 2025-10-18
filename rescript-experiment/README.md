```
npm i
```

Compile rescript into javascript
```
npm run res:dev
```

Run the generated javascript
```
watchexec -r node src/Demo.res.mjs
```

Run tests
```
watchexec -r npm run test
```

TODO figure out typescript stuff
https://rescript-lang.org/docs/manual/v11.0.0/typescript-integration
https://github.com/ocsigen/ts2ocaml/blob/main/docs/rescript.md