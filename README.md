# **[cellnoor](https://scbldata.jax.org)**
[cellnoor](https://scbldata.jax.org) is a web application and RESTful API that aggregates, displays, and allows comprehensive querying of single-cell biological data.
### **Using the API**
First, obtain an API key:
1. visit [https://scbldata.jax.org](https://scbldata.jax.org)
2. sign in if prompted to do so
3. click "profile"
4. click "show API key"

You can interact with the API using:
- **[scamplepy](#scamplepy)** (recommended)
- **[raw HTTP requests](#raw-http-requests)** using [cURL](https://github.com/curl/curl), [httpx](https://www.python-httpx.org) in Python, or whatever language/client you choose
- **[the Rust client](#rust-client)**, though this is not currently recommended due to code organization and discoverability issues
### **scamplepy (recommended)**
#### **Installation**
Currently, `scamplepy` requires python 3.13 or above. You can install it in an isolated environment using [uv](https://docs.astral.sh/uv/) (recommended):
```bash
uv init my-project
cd my-project
uv add git+https://github.com/demhadais/cellnoor#subdirectory=rust/cellnoor/scamplepy
```
or using [pip](https://github.com/pypa/pip):
```bash
python3 -m venv venv
. venv/bin/activate
pip3 install git+https://github.com/demhadais/cellnoor#subdirectory=rust/cellnoor/scamplepy
```
Installation with other tools, like [poetry](https://python-poetry.org), is untested but likely works. After installation, you can use it to query cellnoor like so:
#### **Usage**
```python
import asyncio

from scamplepy import cellnoorClient
from scamplepy.query import Pagination, PersonQuery


async def main():
    # Set up the client
    client = cellnoorClient(
        api_base_url="https://ctscmongo01lp.jax.org/api",
        api_key="api-key",
    )

    # Create an empty query, which does not filter the data but specifies a default
    # sort-order and limit/offset
    uninteresting_query = PersonQuery()

    # You can also create more interesting queries. This one gets everyone whose name
    # contains "ahmed" or "said", leaving all other filters empty
    interesting_query = PersonQuery(
        names=["ahmed", "said"], pagination=Pagination(limit=1000, offset=0)
    )

    # You can serialize the query in multiple ways for inspection or later use
    query_as_json = interesting_query.to_json_string()
    query_as_json_bytes = interesting_query.to_json_bytes()
    query_as_base64_encoded_json = interesting_query.to_base64_json()

    # You can also deserialize a query from multiple JSON representations
    query_from_json = PersonQuery.from_json_string(query_as_json)
    query_from_json_bytes = PersonQuery.from_json_bytes(query_as_json_bytes)
    query_from_base64_encoded_json = PersonQuery.from_base64_json(
        query_as_base64_encoded_json
    )

    # Send the request
    people = await client.list_people(interesting_query)
    for person in people:
        # Get specific pieces of information
        my_name = person.info.summary.name
        my_email = person.info.summary.email

        # You can serialize the response in all the same ways as the query
        response_as_json = person.to_json_string()


if __name__ == "__main__":
    asyncio.run(main())
```
### **Raw HTTP requests**
To query the API, requests need to have an API key placed in the header as `X-API-Key: my-api-key`. They may optionally have a parameter in the query-string called `query`, set to a URL-safe [base64](https://en.wikipedia.org/wiki/Base64)-encoded JSON object containing the actual data.
#### **[cURL](https://curl.se)**
To execute a query for all people whose name contains any of the strings "ahmed" or "said":
```bash
curl --variable 'interesting_query={"names": ["ahmed", "said"]}' --expand-data 'query={{interesting_query:b64}}' --header 'X-API-Key: krabby-patty-secret-formular' --get https://ctscmongo01lp.jax/org/api/people
```
The query is written as JSON, then fed into cURL for base64-encoding and placement into the query string. See the [cURL documentation](https://curl.se/docs/manpage.html) for more information.
#### **[httpx](https://www.python-httpx.org)**
```python
import json
from base64 import urlsafe_b64encode

import httpx

headers = {"X-API-Key": "api-key"}

data = {"names": ["ahmed", "said"]}
# Some units of measure, like the 'µl', are not ASCII
data = json.dumps(data, ensure_ascii=False).encode()
data = urlsafe_b64encode(data)

with httpx.Client() as client:
    people = client.get(
        "https://ctscmongo01lp.jax.org/api/people",
        headers=headers,
        query={"query": data},
    )
```
## **API specification**
The following is a list of endpionts an the default query for each endpoint. Note the following behaviors, which apply to all parameters for all endpoints unless otherwise specified:
- an array means "match **any** of the following values"
- strings are searched as case-insensitive substrings. For example:
  ```json
  {"names": ["ahmed", "said"]}
  ```
  means "look for people whose name contains any of 'ahmed' or 'said' case-insensitively"
- an empty array means "don't filter by this field"
- all parameters are strings
### **Endpoints**
- [`/institutions`](#institutions)
- [`/people`](#people)
- [`/labs`](#labs-incomplete)
- [`/specimens`](#specimens)
- [`/sequencing-runs`](#sequencing-runs-incomplete)
- [`/10x-assays`](#10x-assays)
- [`/multiplexing-tags`](#multiplexing-tags)
- [`/suspensions`](#suspensions-incomplete)
- [`/suspension-pools`](#suspension-pools-incomplete)
- [`/chromium-runs`](#institutions)
- [`/cdna`](#cdna-incomplete)
- [`/libraries`](#libraries-incomplete)
- [`/chromium-datasets`](#chromium-datasets-incomplete)
### **Institutions**
```jsonc
{
  // array of UUIDs
  "ids": [],
  "names": [],
  // array of `OrderBy` objects
  "order_by": [{ "field": "name", "descending": false }],
  // Pagination object
  "pagination": { "limit": 500, "offset": 0 },
}
```
### **People**
```jsonc
{
  // array of UUIDs (encoded as strings)
  "ids": [],
  "names": [],
  "emails": [],
  "orcids": [],
  // array of UUIDs (encoded as strings)
  "ms_user_ids": [],
  // array of `OrderBy` objects
  "order_by": [{ "field": "name", "descending": false }],
  // Pagination object
  "pagination": { "limit": 500, "offset": 0 },
}
```
### **Labs (incomplete)**
```jsonc
{
  // array of UUIDs (encoded as strings)
  "ids": [],
  "names": [],
  // array of `OrderBy` objects
  "order_by": [{ "field": "name", "descending": false }],
  // Pagination object
  "pagination": { "limit": 500, "offset": 0 },
}
```
### **Specimens**
```jsonc
{
  // array of UUIDs (encoded as strings)
  "ids": [],
  "names": [],
  // array of UUIDs (encoded as strings)
  "submitters": [],
  // array of UUIDs (encoded as strings)
  "labs": [],
  // optional ISO8601 formatted datetime string
  "received_before": null,
  // optional ISO8601 formatted datetime string
  "received_after": null,
  "species": [],
  "notes": [],
  "types": [],
  "embedded_in": [],
  "fixatives": [],
  // optional boolean
  "frozen": null,
  // optional boolean
  "cryopreserved": null,
  // optional object of arbitrary keys and values to search
  "additional_data": null,
  // array of `OrderBy` objects
  "order_by": [{ "field": "received_at", "descending": false }],
  // Pagination object
  "pagination": { "limit": 500, "offset": 0 },
}
```
### **Sequencing runs (incomplete)**
```jsonc
{
  // array of UUIDs (encoded as strings)
  "ids": [],
  // array of `OrderBy` objects
  "order_by": [{ "field": "begun_at", "descending": false }],
  // Pagination object
  "pagination": { "limit": 500, "offset": 0 },
}
```
### **10x Assays**
```jsonc
{
  // array of UUIDs (encoded as strings)
  "ids": [],
  "names": [],
  // array of array of strings (see below)
  "library_types": [],
  "sample_multiplexing": [],
  "chemistry_versions": [],
  "chromium_chips": [],
  // array of `OrderBy` objects
  "order_by": [{ "field": "name", "descending": false }],
  // Pagination object
  "pagination": { "limit": 500, "offset": 0 },
}
```
A [10x Genomics Chromium](https://www.10xgenomics.com/platforms/chromium) assay is defined in part by the combination of library types generated. A query like
```jsonc
{
  "library_types": [
    ["antibody_capture", "gene_expression"],
    ["gene_expression"],
  ],
}
```
finds all assays that consist of:
- an antibody capture library and a gene expression library
- just a gene expression library.
Note that the order of libraries does not matter, so
```jsonc
["antibody_capture", "gene_expression"]
```
is equivalent to
```jsonc
["gene_expression", "antibody_capture"]
```
### **Multiplexing tags**
```jsonc
// No query parameters are allowed for this endpoint
{}
```
### **Suspensions (incomplete)**
```jsonc
{
  // array of UUIDs (encoded as strings)
  "ids": [],
  // optional SpecimenQuery object, as shown above
  "specimen": null,
  // array of `OrderBy` objects
  "order_by": [{ "field": "created_at", "descending": false }],
  // Pagination object
  "pagination": { "limit": 500, "offset": 0 },
}
```
### **Suspension pools (incomplete)**
```jsonc
{
  // array of UUIDs (encoded as strings)
  "ids": [],
  // array of `OrderBy` objects
  "order_by": [{ "field": "pooled_at", "descending": false }],
  // Pagination object
  "pagination": { "limit": 500, "offset": 0 },
}
```
### **Chromium runs**
```jsonc
{
  // array of UUIDs (encoded as strings)
  "ids": [],
  "readable_ids": [],
  // optional TenxAssayQuery object, as shown above
  "assay": null,
  // optional ISO8601 formatted datetime string
  "run_before": null,
  // optional ISO8601 formatted datetime string
  "run_after": null,
  // optional boolean
  "succeeded": null,
  "notes": [],
  // array of `OrderBy` objects
  "order_by": [{ "field": "run_at", "descending": false }],
  // Pagination object
  "pagination": { "limit": 500, "offset": 0 },
}
```
### **cDNA (incomplete)**
```jsonc
{
  // array of UUIDs (encoded as strings)
  "ids": [],
  // array of `OrderBy` objects
  "order_by": [{ "field": "prepared_at", "descending": false }],
  // Pagination object
  "pagination": { "limit": 500, "offset": 0 },
}
```
### **Libraries (incomplete)**
```jsonc
{
  // array of UUIDs (encoded as strings)
  "ids": [],
  "library_types": [],
  // array of `OrderBy` objects
  "order_by": [{ "field": "prepared_at", "descending": false }],
  // Pagination object
  "pagination": { "limit": 500, "offset": 0 },
}
```
### **Chromium Datasets (incomplete)**
```jsonc
{
  // array of UUIDs (encoded as strings)
  "ids": [],
  "names": [],
  // array of UUIDs (encoded as strings)
  "lab_ids": [],
  // optional ISO8601 formatted datetime string
  "delivered_before": null,
  // optional ISO8601 formatted datetime string
  "delivered_after": null,
  // optional TenxAssayQuery object, as shown above
  "tenx_assay": null,
  // optional SpecimenQuery object, as shown above
  "specimen": null,
  // array of `OrderBy` objects
  "order_by": [{ "field": "delivered_at", "descending": false }],
  // Pagination object
  "pagination": { "limit": 500, "offset": 0 },
}
```
