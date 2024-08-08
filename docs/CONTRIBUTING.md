## Integration tests

TO BE DONE

### Test the service in local environment

First launch Docker Compose (`docker-compose up`) in order to start Mockoon, then run the following command:
    
```bash
> cargo run -- --s3-dry-run --cerved-api-base-url http://localhost:3001 --cerved-oauth-base-url http://localhost:3001 --cerved-oauth-username <CERVED_OAUTH_USERNAME> --cerved-oauth-password <CERVED_OAUTH_PASSWORD> --qrp-bucket-name <QRP_BUCKET_NAME>
```

Now you can test the service using the following curl command:

```bash
> curl -X POST http://localhost:8080/qrp/012345?user=test&maxTtl=1
```
