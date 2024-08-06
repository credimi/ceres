# Overview

The service has a simple REST API that allows to get from Cerved the QRP data for a given VAT number, both in XML and PDF format.
The retrieved data is then directly stored on S3.

At the moment the following endpoints are available:
- `/qrp/{vat_number}` (legacy endpoint)
