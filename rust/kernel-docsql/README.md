# DocsQL

DocsQL is a domain-specific language (DSL) for querying graphs of scientific documents that is designed to be easier for both humans and LLMs to write compared to raw Cypher queries. It provides an intuitive interface for exploring and analyzing document structures, content relationships, and metadata.

## Purpose

DocsQL enables users to query document databases containing scientific papers, articles, and other structured documents. The language abstracts the complexity of graph database queries while maintaining the full power of the underlying Cypher query language. It's particularly useful for:

- **Document analysis**: Finding specific content types, sections, or elements within documents
- **Research workflows**: Querying papers by authors, references, sections, or content similarity
- **Content discovery**: Searching for related content using semantic similarity or full-text search
- **Structural analysis**: Exploring document hierarchies and relationships between elements
- **AI-assisted querying**: Providing an LLM-friendly syntax for automated document analysis

## Key Features

- **Human-readable syntax**: Natural language-like queries that are intuitive to write and understand
- **Type-aware querying**: Built-in support for document node types (paragraphs, figures, tables, etc.)
- **Flexible filtering**: Rich set of operators for property-based filtering
- **Semantic search**: Vector similarity search using embeddings
- **Full-text search**: Advanced text search with ranking
- **Positional queries**: Find content relative to current document position (document database only)
- **Subquery support**: Complex nested queries with counting and existence checks
- **Method chaining**: Fluent interface for building complex queries

## Database Targets

DocsQL queries can be run against different document databases:

- **`document`** - The current document's database containing all nodes and content. Supports positional queries (`above`/`below`) relative to the query's position within the host document.
- **`workspace`** - The workspace's document database containing multiple documents. Used for cross-document analysis and research across collections.
- **`openalex`** - Query the OpenAlex database of scholarly works, authors, and institutions. Provides access to millions of research papers with metadata, citations, and author information.
- **`github`** - Query GitHub repositories, files, and users to discover open-source research materials like Jupyter notebooks, Quarto documents, and scientific code.

#### External API Limitations

While DocsQL provides convenient interfaces to external APIs, there are some limitations:

**OpenAlex:**
- **Reference filtering**: References can only be filtered by count (`...references(* > 10)`) or specific work IDs. Filtering by reference properties like title or topic is not supported.
- **Operator compatibility**: The `>=` and `<=` operators are automatically converted to equivalent `>` and `<` expressions (e.g., `>=10` becomes `>9`) due to API limitations.
- **Semantic search**: The `like` method for semantic similarity is not available; use `search` for text-based matching instead.

**GitHub:**
- **API rate limits**: 5000 requests/hour (authenticated) or 60/hour (unauthenticated)
- **Search result limits**: Maximum 1000 results per query
- **File size limits**: Content search limited to files <384KB
- **Required search terms**: All queries require a search parameter; empty queries use `*` wildcard

**Shared Limitations:**
- **Pagination approximation**: `skip` and `limit` operations are approximated using API pagination. When `skip + limit` exceeds API per-page limits (100 for GitHub, 200 for OpenAlex), results may be incomplete.
- **No semantic search**: The `like` method is not supported; use `search` for text-based matching.

### Free Functions vs Database Methods

DocsQL provides convenient "free functions" that are shortcuts for querying the current document:

```docsql
// These free functions target the document database
paragraphs()           // Equivalent to: document.paragraphs()
figures()             // Equivalent to: document.figures()
mathBlocks()          // Equivalent to: document.mathBlocks()

// Singular versions automatically add .limit(1)
paragraph(.text ^= "Introduction")  // Equivalent to: document.paragraphs(.text ^= "Introduction").limit(1)
figure(1)                          // Equivalent to: document.figures(.label == "1").limit(1)
table(.caption $= "results")       // Equivalent to: document.tables(.caption $= "results").limit(1)
```

**Important**: Positional keywords (`above`, `below`) only work with the `document` database since they reference the position within the host document where the query is executed.

## Quick Start

Here are the most common DocsQL query patterns to get you started:

```docsql
// Get all paragraphs in current document
paragraphs()

// Find a specific figure by label
figure(1)

// Search for content
paragraphs(search = "machine learning")

// Find content above current position
paragraphs(above).limit(3)

// Query workspace for articles by author
workspace.articles(...authored_by(.name ^= "Smith"))

// Query GitHub for Jupyter notebooks
github.files(.extension == "ipynb", search = "machine learning")

// Query OpenAlex for recent papers
openalex.works(.year > 2020, search = "neural networks")
```

## Syntax Overview

### Basic Query Pattern
```docsql
[database.]nodeType([positional|filters])[.methods()]
```

### Filter Syntax
```docsql
// Property filters use dot notation
.propertyName operator value

// Examples:
.text == "Introduction"     // equality
.position > 100            // numeric comparison  
.title ^= "Chapter"        // starts with
.content ~= "pattern.*"    // regex match
.tags in ["AI", "ML"]      // list membership
```

### Common Methods
```docsql
.limit(n)                  // limit results
.skip(n)                   // skip first n results
.sort("property")       // sort results
.search("terms")           // full-text search
.like("text")              // semantic similarity
```

### Subquery Pattern
```docsql
// Use ... for existence/count subqueries
nodeType(...relatedType(filters))
articles(...authored_by(.name ^= "Jane"))
```

## Query Examples

### Simple Queries

```docsql
// Get content by type
paragraphs()                    // All paragraphs
headings()                      // All headings
codeChunks()                    // All code chunks
figures()                       // All figures

// Get specific items (singular = limit 1)
paragraph(.text == "Abstract")  // First paragraph with text "Abstract"
heading(.level == 1)           // First level 1 heading
figure(1)                      // Figure with label "1"
```

### Filtering and Search

```docsql
// Property-based filtering
paragraphs(.text ^= "The study")        // Paragraphs starting with "The study"
figures(.position > 1000)               // Figures after position 1000
sections(.sectionType == "Methods")     // Methods sections

// Text search
paragraphs(search = "neural networks")               // Full-text search
paragraphs(like = "machine learning applications")   // Semantic similarity

// Combined filters
sections(.sectionType == "Results", search = "significant")
```

### Positional Queries (Document Only)

Positional queries find content relative to where the query appears in the document:

```docsql
// Find content above/below current position
figures(above)                  // Figures above current position
paragraphs(below).limit(5)      // Next 5 paragraphs below current position
tables(above).first()           // Most recent table above current position

// Note: These don't work on workspace queries
// workspace.articles(above)    // ❌ Error: positional queries need document context
```

### Cross-Document Queries (Workspace)

```docsql
// Query across multiple documents
workspace.articles()                                    // All articles in workspace
workspace.articles(.title ^= "Deep Learning")          // Articles with titles starting with "Deep Learning"
workspace.articles(...authored_by(.name == "Jane Smith"))  // Articles by Jane Smith
workspace.articles(...references(* > 20))              // Articles with more than 20 references
workspace.articles(...cites(* > 20))                   // Articles that cite more than 20 works (alias for references)
workspace.articles(...citedBy(* > 100))                // Articles cited more than 100 times
```

### Complex Queries with Subqueries

```docsql
// Existence subqueries (does related content exist?)
sections(...codeChunks(.programmingLanguage == "python"))  // Sections containing Python code
articles(...authored_by(.affiliations(.name $= "University"))) // Articles with university-affiliated authors

// Count subqueries (how many related items?)
articles(...authored_by(* > 3))                 // Articles with more than 3 authors
sections(...paragraphs(* == 1))             // Sections with exactly 1 paragraph
workspace.articles(...references(* <= 10))  // Articles with 10 or fewer references
workspace.articles(...cites(* >= 15))       // Articles that cite 15 or more works
workspace.articles(...citedBy(* >= 50))     // Articles cited 50 or more times

// GitHub subqueries
github.repositories(...topics(.name == "data-science"))     // Repositories tagged with data science
github.repositories(...owners(.name ~= ".*University.*"))   // Repositories owned by universities
github.files(.extension == "qmd", ...repositories(.language == "python"))  // Quarto files in Python repos

// Query object subqueries (advanced citation analysis)
openalex.articles(...citedBy(openalex.articles(search = "frogs").limit(3)))
// Find articles cited by research about frogs

openalex.works(...citedBy(openalex.articles(...authored_by(.orcid == "0000-0002-1825-0097")).limit(2)))
// Find works cited by a specific researcher (using ORCID)

openalex.articles(...citedBy(openalex.works(.doi == "10.1038/nature12373").limit(1)))
// Find articles cited by a specific Nature paper (using DOI)

openalex.articles(...citedBy(openalex.articles(...authored_by(.name == "Jane Smith")).limit(4)))
// Find articles cited by papers authored by Jane Smith

openalex.works(...citedBy(openalex.articles(.year == 2023).limit(3)))
// Find works cited by recent 2023 articles

openalex.works(...citedBy(openalex.articles(.cited_by_count > 100).limit(5)))
// Find works cited by highly-cited papers (>100 citations)

openalex.articles(...citedBy(openalex.works(.year > 2020).limit(3)))
// Find articles cited by recent works since 2020

openalex.works(...citedBy(workspace.articles().limit(5)))
// Find works cited by articles in your workspace

// Chained subqueries
workspace.articles(...authored_by(.name ^= "John").affiliations(.name $= "MIT"))
// Articles by authors named John who are affiliated with institutions ending in "MIT"
```

## Advanced Features

### Variable Assignment

```docsql
// Store query results in variables
let recentFigures = figures(.position > 2000)
let workspaceArticles = workspace.articles(.datePublished > "2023-01-01")

// Use variables in other queries
recentFigures.limit(3)
workspaceArticles.count()
```

### Method Chaining

```docsql
// Chain multiple operations
paragraphs()
  .skip(10)
  .limit(5)
  .sort("position")

// Complex filtering and processing
workspace.articles()
  .where(...authored_by(.name ^= "Smith"))
  .sort("datePublished", "DESC")
  .limit(10)
  .select("title", "datePublished")
```

### Custom Cypher

For advanced use cases, you can specify raw Cypher on any database:

```docsql
// Custom query on current document
document.cypher("MATCH (n:Paragraph) WHERE n.text CONTAINS 'AI' RETURN n LIMIT 5")

// Custom query on workspace
workspace.cypher("MATCH (a:Article)-[:authors]->(p:Person) RETURN a.title, p.name LIMIT 10")
```

### Combining Results

```docsql
// Combine results from multiple queries
let allCode = combine(codeBlocks(), codeChunks(), codeExpressions())

// Union queries
let importantSections = methods().union(results())
```

## Reference

### GitHub-Specific Queries

**Entity Types:**
- `github.repositories()` / `github.repos()` - GitHub repositories
- `github.files()` / `github.code()` - Files and code within repositories
- `github.users()` - GitHub users and organizations

**Target File Types for Research:**
- **Jupyter Notebooks**: `.extension == "ipynb"`
- **Quarto Documents**: `.extension == "qmd"`
- **MyST Markdown**: `.extension == "md"` with MyST syntax
- **Research Formats**: `.extension in ["tex", "bib", "rmd"]`

**Common GitHub Filters:**
```docsql
// Repository properties
.language == "python"              // Programming language
.stars > 100                       // Star count
.size < 1000                       // Repository size in KB
.created > "2023-01-01"           // Creation date
.updated > "2023-01-01"           // Last update
.license == "mit"                  // License type

// File properties  
.filename == "README.md"           // Specific filename
.path ~= ".*docs.*"               // Path pattern
.extension == "py"                // File extension

// Search and content
search = "machine learning"        // Text search in content
.content ~= "import pandas"       // Content pattern matching
```

**GitHub Query Examples:**
```docsql
// Find popular Python repositories with Jupyter notebooks
github.repositories(.language == "python", .stars > 100)
  .files(.extension == "ipynb")

// Find Quarto documents in data science repositories
github.repositories(...topics(.name == "data-science"))
  .files(.extension == "qmd")

// Find research repositories by academic institutions
github.repositories(...owners(.name ~= ".*University.*"))
  .search("research")

// Count notebooks in bioinformatics repositories
github.repositories(...topics(.name == "bioinformatics"))
  .files(.extension == "ipynb").count()

// Find recently updated research documents
github.files(.extension in ["ipynb", "qmd", "md"])
  .repositories(.updated > "2023-01-01")
```

### Available Node Types

All node type functions are available as both singular (returns first match) and plural (returns all matches) forms. They can be used as free functions (targeting the `document` database) or as methods on specific databases.

**Content Elements:**
- `paragraph` / `paragraphs` - Text paragraphs
- `heading` / `headings` - Document headings
- `section` / `sections` - Document sections
- `sentence` / `sentences` - Individual sentences
- `list` / `lists` - Lists and list items

**Code Elements:**
- `codeBlock` / `codeBlocks` - Static code blocks
- `codeChunk` / `codeChunks` / `chunk` / `chunks` - Executable code chunks
- `codeExpression` / `codeExpressions` / `expression` / `expressions` - Inline executable code
- `codeInline` / `codeInlines` - Inline code snippets

**Mathematical Content:**
- `mathBlock` / `mathBlocks` - Block-level mathematical expressions
- `mathInline` / `mathInlines` - Inline mathematical expressions

**Media Elements:**
- `image` / `images` - Image objects
- `audio` / `audios` - Audio objects  
- `video` / `videos` - Video objects

**Structured Elements:**
- `table` / `tables` - Tables
- `figure` / `figures` - Figures
- `equation` / `equations` - Labeled equations
- `admonition` / `admonitions` - Admonitions/callouts
- `claim` / `claims` - Claims

**Metadata:**
- `author` / `authors` / `person` / `people` - Authors and persons
- `organization` / `organizations` - Organizations and affiliations
- `reference` / `references` - Citations and references
- `variable` / `variables` - Document variables

**Section Types:**
- `introduction` - Introduction sections
- `methods` - Methods sections  
- `results` - Results sections
- `discussion` - Discussion sections

### Operators

**Comparison Operators:**
- `==` or `=` - Equality
- `!=` - Inequality  
- `<` - Less than
- `<=` - Less than or equal
- `>` - Greater than
- `>=` - Greater than or equal

**String Operators:**
- `~=` - Regex match
- `~!` - Regex non-match
- `^=` - Starts with
- `$=` - Ends with

**Collection Operators:**
- `in` - Value is in collection
- `has` - Collection contains value

**Special Operators:**
- `*` - Count operator (for subqueries only)

### Query Methods

**Core Query Building:**
- `.match(pattern)` - Specify custom Cypher MATCH pattern
- `.where(condition)` / `.and(condition)` - Add AND conditions
- `.or(condition)` - Add OR conditions
- `.return(expression)` - Specify return clause

**Result Processing:**
- `.count()` - Return count of matching nodes
- `.select(columns...)` - Select specific properties for tabular output
- `.first()` / `.one()` - Get first result
- `.last()` - Get last result
- `.all()` - Get all results (default)
- `.slice(start, end)` - Get slice of results

**Sorting and Limiting:**
- `.sort(property, direction)` - Sort results
- `.skip(n)` - Skip first n results
- `.limit(n)` - Limit to n results  
- `.sample(n)` - Random sample of n results (default 10)

**Combining Queries:**
- `.union(otherQuery, all=false)` - Union with another query
- `combine(query1, query2, ...)` - Combine multiple query results

**Output Control:**
- `.out(format)` - Specify output format
- `.explain()` - Show generated Cypher query
- `.text()` - Get text representation of results

## Development

### Testing

The `golden.rs` tests, test that:

- **Query compilation**: DocsQL queries are correctly translated to Cypher or external API calls
- **API integration**: Generated URLs for external services (like OpenAlex) are valid and functional
- **Consistency**: Query results match expected outputs across different scenarios

Test case files are located in `tests/` with extensions:

- `.cypher` - Tests that generate Cypher queries for document databases
- `.openalex` - Tests that generate OpenAlex API requests
- `.github` - Tests that generate GitHub API requests

#### Running Tests

Run all tests:

```bash
cargo test golden
```

Run tests for a specific pattern:

```bash
GLOB="*.github" cargo test golden
```

Run a specific test file:

```bash
GLOB="subquery.openalex" cargo test golden
```

Skip HTTP requests (faster, offline-friendly):

```bash
NO_HTTP=1 cargo test golden
```

Update test expectations:

```bash
UPDATE_GOLDEN=1 cargo test golden
```

Combine options:

```bash
GLOB="*.github" NO_HTTP=1 cargo test golden
```

#### Environment Variables

- **`GLOB`** - Run tests matching a specific glob pattern (e.g., `*.github`, `subquery.openalex`, `*basic*`)
- **`NO_HTTP`** - Skip HTTP validation for OpenAlex and GitHub URLs (useful for offline testing)
- **`UPDATE_GOLDEN`** - Update test files with new expected outputs instead of asserting
