TODO TUI dashboard, watcher

TODO cloudformation deployments

```
# ~/.swift-aws-tool/config.yaml
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
# eval on the first invocation blocks typing the MFA code
swift run Experiment env --profile main && eval $(swift run Experiment env --profile main)
swift run Experiment env --profile sdlc && eval $(swift run Experiment env --profile sdlc)
```

```
swift run Experiment list-clusters --profile main --region us-east-1
swift run Experiment list-clusters --profile main --region eu-central-1
```

```
swift run Experiment describe-cluster --profile main --region us-east-1 --cluster-name broker-nonprod --filter broker-dev
swift run Experiment describe-cluster --profile main --region us-east-1 --cluster-name broker-nonprod --filter broker-qa
swift run Experiment describe-cluster --profile main --region us-east-1 --cluster-name broker-stg
swift run Experiment describe-cluster --profile main --region us-east-1 --cluster-name broker-sandbox
swift run Experiment describe-cluster --profile main --region us-east-1 --cluster-name broker-prod
swift run Experiment describe-cluster --profile main --region eu-central-1 --cluster-name broker-stg-eu
swift run Experiment describe-cluster --profile main --region eu-central-1 --cluster-name broker-prod-eu
```