---
structuring:
- sections-to-abstract
- sections-to-references
- headings-to-sections
---

This tests that Abstract and References sections are extracted as document metadata while remaining sections get proper structure.

# Introduction

This section should remain in the main content and be wrapped in a section element.

# Abstract

This content should be extracted as document metadata and removed from the main content. The Abstract heading and this paragraph should become the document's abstract property.

# Methods

This section should remain in the main content and be properly structured as a section.

# Results

The results section should also remain and be wrapped in a section element.

# References

Smith, J. (2023). Machine Learning Applications. Academic Press.

Jones, A. (2022). Data Analysis Methods. Science Publications.

Wilson, B. (2021). Statistical Approaches. Research Press.

# Conclusion

The References section above should be extracted as document metadata, and the reference list should be processed into structured references while being removed from the main content.
