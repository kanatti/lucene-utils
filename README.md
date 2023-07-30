# Lucene Utils

Tooling to work with lucene index files. This is an experimental project and in verly early stages.

## Sample Parsed SegmentInfo

```
Ok(
    SegmentInfos {
        version: 4,
        index_created_version: 10,
        generation: 1,
        lucene_version: (10,0,0),
        id: [86,20,232,254,232,129,34,124,90,106,95,58,227,218,195,10],
        counter: 1,
        num_segments: 1,
        min_segment_lucene_version: (10,0,0),
        segments: [
            SegmentInfo {
                name: "_0",
                id: [86,20,232,254,232,129,34,124,90,106,95,58,227,218,195,7],
                version: (10,0,0),
                min_version: Some((10,0,0)),
                doc_count: 2,
                is_compound_file: true,
                diagnostics: {
                    "java.vendor": "Amazon.com Inc.",
                    "source": "flush",
                    "os.arch": "aarch64",
                    "os.version": "13.0",
                    "java.runtime.version": "17.0.6+10-LTS",
                    "os": "Mac OS X",
                    "timestamp": "1690600805698",
                    "lucene.version": "10.0.0",
                },
                files: {
                    "_0.cfe",
                    "_0.cfs",
                    "_0.si",
                },
                attributes: {
                    "Lucene90StoredFieldsFormat.mode": "BEST_SPEED",
                },
                sort_fields: [],
            },
        ],
    },
)
```