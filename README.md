<div align="center">
	<img
	width="80"
	src="/images/mulesoft-logo.png">
	<h1>Open API Data Loss Prevention Policy</h1>
</div>

<h4 align="center">
	<a href="#overview">Overview</a> |
	<a href="#try-it">Try It</a> |
  <a href="#make-command-reference">Make Reference</a>
</h4>

## Overview

This policy was created with the Flex Gateway Policy Development Kit (PDK). To find the complete PDK documentation, see [PDK Overview](https://docs.mulesoft.com/pdk/latest/policies-pdk-overview) on the MuleSoft documentation site.

The component has the following properties that can be set at design time in App Builder by an administrator

| Property                                   | Description                                                                                                 | Type                                   |
| ------------------------------------------ | ------------------------------------------------------------------------------------------------------------| -------------------------------------- |
| `presidio-analysis-service`                | The Presidio Analyze service location running in Docker                                                     | String    
| `langauge`                                 | The language used by Presidio in ISO-639_1 format                                                           | String    
| `score_threshold`                          | The score threshold in Presidio for it to be flagged as sensitive (0-1)                                     | String    
| `entities`                                 | An array of entities to look for in the OpenAI request                                                      | Array    
| `action`                                   | Log - Log sensitive data but continue or Reject - if sensitive data found return 401 (Unauthorized)         | String    


### Example

When calling the OpenAI API a user will potentially include sensitve data in the prompt.

However, with Flex gateway being used and openai.api.com as the upstream api we can intercept the request and use pii checking utilities to look for sensitive data. In this case Microsoft's Presidio.

This policy assumes you have already added your OpenAI API key to the request header. You can do this in your http client or use the policy [Open API Key Management Policy] (https://github.com/deejay-hub/openai-api-key-mgmt)

```
>  curl http://localhost:8081/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-3.5-turbo",
    "messages": [
      {
        "role": "system",
        "content": "You are an assistant, skilled in explaining MuleSoft concepts with creative flair. Keep responses free from bias and without obsenities"
      },
      {
        "role": "user",
        "content": "create a poem about Flex Gateway in less than 20 words"
      }
    ]
  }'
  
```

## Try It
These setup steps use Flex Gateway in connected mode running locally on a Mac. Make sure you have docker desktop installed.

### Flex Gateway
Step 1. Open Anypoint Platform and head to Runtime Manager. Select Flex Gateways then choose Container -> Docker.

<p align="center">
  <img alt="gateway-setup" src="images/add-gateway.png">
</p>

Step 2. Pull the flex gateway image and start the gateway on port 8081.

a) Pull the latest image
```
docker pull mulesoft/flex-gateway
```
b) Create a new directory then register the Flex Gateway to Anypoint Platform replacing `gateway-name` with your own value.
```
docker run --entrypoint flexctl -u $UID \
  -v "$(pwd)":/registration mulesoft/flex-gateway \
  registration create --organization=adfa825c-fc0d-4ba3-a5ba-ab35a7194a39 \
  --token=8eb976e5-9ab1-4048-bdd6-89504008632a \
  --output-directory=/registration \
  --connected=true \
  <gateway-name>
```
c) Start the gateway
```
docker run --rm \
  -v "$(pwd)":/usr/local/share/mulesoft/flex-gateway/conf.d \
  -p 8081:8081 \
  mulesoft/flex-gateway
```

Step 3. Head to API Manager in Anypoint. Select `Add API -> Add new API`.

Step 4. Make sure Flex Gateway is selected as the runtime then select the Flex Gateway you created in Step 2. Click Next.

Step 5. Select Create new API. Choose `OpenAI API` as the name and and `open-ai-api` as the asset id. Click Next.

Step 6. Select Port `8081` and change the base path to `/flex-api`. Click Next.

Step 7. Enter `https://openai.api.com/v1` as the upstream URL

<p align="center">
  <img alt="policy-config" src="images/api-settings.png">
</p>

Test the configuration by calling the Flex Gateway endpoint to see that you get a response from OpenAI. Note that since we don't include the OpenAI API key we will get a 401 unauthorised at this stage. Note that you will need to restrict the response payload so use terms like 'in less than 20 words' to get OpenAI to return a consumable response.

```
curl -X POST http://localhost:8081/flex-api/chat/completions   -H "Content-Type: application/json"  -d '{
    "model": "gpt-3.5-turbo",
    "messages": [
      {
        "role": "system",
        "content": "You are an assistant, skilled in explaining MuleSoft concepts with creative flair. Keep responses free from bias and without obsenities."
      },
      {
        "role": "user",
        "content": "Compose a poem in less than 10 words."
      }
    ]
  }'
```
Step 8. After downloading the policy use `make build` then `make release`.

Step 9. Apply the policy in API Manager to the API created in Step 3. In the openai-api-key section enter a valid key from OpenAI.

<p align="center">
  <img alt="policy-config" src="images/policy-config.png">
</p>

Step 10. Verify that the policy now successfully gets a response.

## Make command reference
This project has a Makefile that includes different goals that assist the developer during the policy development lifecycle.

*For more information about the Makefile, see [Makefile](https://docs.mulesoft.com/pdk/latest/policies-pdk-create-project#makefile).*

### Setup
The `make setup` goal installs the Policy Development Kit internal dependencies for the rest of the Makefile goals.
Since these dependencies are provided by the Anypoint Platform, it requires the user to be authenticated with a set of valid Anypoint credentials.

*For more information about `make setup`, see [Setup the PDK Build environment](https://docs.mulesoft.com/pdk/latest/policies-pdk-create-project#setup-the-pdk-build-environment).*

### Build asset files
The `make build-asset-files` goal generates all the policy asset files required to build, execute, and publish the policy. This command also updates the `config.rs` source code file with the latest configurations defined in the policy definition.

*For more information about creating a policy definition, see [Defining a Policy Schema Definition](https://docs.mulesoft.com/pdk/latest/policies-pdk-create-schema-definition).*

*For more information about `make build-asset-files`, see [Compiling Custom Policies](https://docs.mulesoft.com/pdk/latest/policies-pdk-compile-policies).*

### Build
The `make build` goal compiles the WebAssembly binary of the policy.
Since the source code must be in sync with the policy definition configurations, this goal runs the `build-asset-files` before compiling.

*For more information about `make build`, see [Compiling Custom Policies](https://docs.mulesoft.com/pdk/latest/policies-pdk-compile-policies).*

### Run
The `make run` goal provides a simple way to execute the current build of the policy in a Docker containerized environment. In order to run this goal, the `playground/config` directory must contain a set of files required for executing the policy in a Flex Gateway instance:
- A `registration.yaml` file generated by performing a Flex Gateway registration in Local Mode. If you already have an instance registered in Local mode, you can reuse the registration file you have and copy it in the `playground/config` folder.
Otherwise, to complete the registration we recommend using the Anypoint Platform:
    1. Go to `Runtime Manager`
    2. Navigate to the `Flex Gateway` tab
    3. Click the `Add Gateway` button
    4. Select `Docker` as your OS and copy the registration command replacing `--connected=true` to `--connected=false`.
    5. Paste the command and run it in the `playground/config` directory.

- An `api.yaml` file updated with the desired policy configuration. This file also supports adding other policies to be applied along the one being developed.

The `playground/config` directory can also contain other resource definitions, such as accessory services used by the policy (Eg. a remote authentication service).

*For more information about `make run`, see [Debugging Custom Policies Locally with PDK](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).*

### Test
The `make test` goal runs unit tests and integration tests. Integration tests are placed in the `tests` directory and are configured with the files placed at the
`tests/<module-name>/<test-name>` directory.

*For more information about writing integration tests, see [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).*

### Publish
The `make publish` goal publishes the policy asset in Anypoint Exchange, in your configured Organization.

Since the publish goal is intended to publish a policy asset in development, the _assetId_ and name published will explicitly say `dev`, and the versions published will include a timestamp at the end of the version. Eg.
- groupId: your configured organization id
- visible name: _{Your policy name} Dev_
- assetId: _{your-policy-asset-id}-dev_
- version: _{your-policy-version}-20230618115723_

*For more information about publishing policies, see [Uploading Custom Policies to Exchange](https://docs.mulesoft.com/pdk/latest/policies-pdk-publish-policies).*

### Release
The `make release` goal also publishes the policy to Anypoint Exchange, but as a ready for production asset. In this case, the groupId, visible name, assetId and version will be the ones defined in the project.

*For more information about releasing policies, see [Uploading Custom Policies to Exchange](https://docs.mulesoft.com/pdk/latest/policies-pdk-publish-policies).*


### Policy Examples

The PDK provides provides a set of example policy projects to get started creating policies and using the PDK features. To learn more about these examples see [Custom policy Examples](https://docs.mulesoft.com/pdk/latest/policies-pdk-policy-templates).
