# **[cellnoor](https://cellnoor.jax.org)**
[cellnoor](https://cellnoor.jax.org) is a web application and RESTful API that aggregates, displays, and allows comprehensive querying of single-cell biological data.
## **Authentication**
First, obtain an API key:
1. Visit [https://cellnoor.jax.org](https://cellnoor.jax.org)
2. Sign in if prompted to do so
3. Click your name in the top right corner
4. Click API keys
5. Generate one or more API keys

You can now put the API key in the header as `X-API-Key: <your API key>` when making requests.
## **Making API Requests**
### Using [cURL](https://curl.se)
To execute a query for all people whose name is any of the strings "ahmed" or "said", limiting to 10 results:
```bash
curl --globoff 'https://cellnoor.jax/org/api/people?filter[names][0]=ahmed&filter[names][0]=said&limit=10'
```
The `globoff` flag is used to turn off `cURL`'s behavior with brackets. See the [manpage](https://curl.se/docs/manpage.html) for more information.
### Using [httpx](https://www.python-httpx.org) and [qs-codec](https://techouse.github.io/qs_codec/)
```python
import httpx
import qs_codec as qs

headers = {"X-API-Key": "api-key"}

query = qs.encode({"filter": {"names": ["ahmed", "said"]}, "limit": 10})

with httpx.Client() as client:
    people = client.get(
        f"https://cellnoor.jax.org/api/people?{query}",
        headers=headers,
        query=query,
    )
```
## **API specification**
The following is a list of endpoints and the default query for each endpoint. Note the following behaviors, which apply to all parameters for all endpoints unless otherwise specified:
- an array means "match **any** of the following values"
- strings are matched case-insensitively. For example:
  ```
  ?filter[names][0]=ahmed&filter[names][1]=said
  ```
  means "look for people whose name is any of 'ahmed' or 'said' case-insensitively"

- To search substrings, use `%`, in accordance with the [Postgres `like` operator](https://www.postgresql.org/docs/current/functions-matching.html#FUNCTIONS-LIKE). For example:
  ```
  ?filter[names][0]=ahmed%
  ```
  searches for anyone whose name starts with "ahmed"

Query strings must be encoded as in the same way the NPM library [`qs`](https://www.npmjs.com/package/qs) encodes them. To see available filters and fields by which to order the results by, see [`./pkgs/cellnoor-types`](./pkgs/cellnoor-types). For example, you can query institutions using the schema defined by [`InstitutionQuery`](./pkgs/cellnoor-types/InstitutionQuery.ts). You can also depend on this package in a `node` script to enforce type-safety in your own programs (as [`cellnoor-ui`](./cellnoor-ui) does).
### **Endpoints**
- `/institutions`
- `/people`
- `/labs`
- `/specimens`
- `/10x-assays`
- `/sequencing-runs`
- `/multiplexing-tags`
- `/suspensions`
- `/suspension-pools`
- `/chromium-runs`
- `/gem-pools`
- `/cdna`
- `/libraries`
- `/chromium-datasets`
