# all-MiniLM-L6-v2

The bundled ONNX model and tokenizer are from `sentence-transformers/all-MiniLM-L6-v2`, used through Grafeo's pinned `MiniLmL6v2` embedding profile. The upstream model is distributed under the Apache License 2.0. Source: <https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2>.

The packaged ONNX graph removes the external `token_type_ids` input and generates the equivalent all-zero tensor internally so it matches Grafeo's documented `input_ids` plus `attention_mask` interface.
