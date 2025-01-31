TODO plan

goal 1:
- cli only, tui dashboard
- periodically check ecs cluster status

goal 2:
- more configurable tui dashboard
- check on other statuses, e.g. cloudformation stack updates


```
# config/config.yaml
profiles:
  main:
    awsAccessKey: <redacted>
    awsSecretAccessKey: <redacted>
    serialNumber: arn:aws:iam::235831761869:mfa/my_phone
  sdlc:
    awsAccessKey: <redacted>
    awsSecretAccessKey: <redacted>
    serialNumber: arn:aws:iam::051394297129:mfa/Jeffs_phone
```

```
swift run Experiment env --profile main
swift run Experiment env --profile sdlc

TODO if it needs to prompt for fresh session token the eval doesn't actually work
eval $(swift run Experiment env --profile main)
eval $(swift run Experiment env --profile sdlc)
```