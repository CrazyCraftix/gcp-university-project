# gcp translate app with caching
translate app that is deployed to gcp
uses redis database as cache

## deployment
- set github actions secret `DEPLOY_STATE_PASSWORD` to secure password
  this is used to encrypt the terraform state and the ssh keypair before storing it on the `deploy-state` branch
- create gcp service account
  it needs permissions to
  - use google translate
  - create virtual machines
  - create the redis database
- set github actions secret `GOOGLE_CREDENTIALS` to the json credentials of the gcp service account
- run `deploy to gcp` workflow - this will take a while
- run `undeploy from gcp` to undeploy
