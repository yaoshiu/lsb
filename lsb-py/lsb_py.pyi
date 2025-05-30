def embed(
    input: bytes,
    extension: str,
    container: bytes,
    lsbs: int = 1,
    hash: str = "blake3",
    seed: int = 42,
    format: str = "png",
) -> bytes:
    """Embeds a payload into a container image.

    Args:
        input (bytes): The payload to embed.
        extension (str): The extension of the payload.
        container (bytes): The container image.
        lsbs (int): The number of least significant bits to use.
        hash (str): The hash algorithm to use.
        seed (int): The seed for the random number generator.
        format (str): The format of the container image.

    Returns:
        bytes: The container image with the embedded payload.
    """
    ...


def extract(
    input: bytes,
    lsbs: int = 1,
    seed: int = 42,
) -> tuple[bytes, str]:
    """Extracts a payload from a container image.

    Args:
        input (bytes): The container image with the embedded payload.
        lsbs (int): The number of least significant bits used for embedding.
        seed (int): The seed for the random number generator used for embedding.

    Returns:
        tuple[bytes, str]: A tuple containing the extracted payload and its extension.
    """
    ...