"""
OmenDB Python Client SDK.

This module provides a Python client for interacting with OmenDB
through the REST API.
"""

import requests
import logging
import numpy as np
from typing import Dict, List, Optional, Union, Tuple


class OmenDBClient:
    """
    Python client for OmenDB vector database.

    This client provides a simple interface for interacting with OmenDB
    using the REST API.
    """

    def __init__(self, host: str = "127.0.0.1", port: int = 8080):
        """
        Initialize the OmenDB client.

        Args:
            host: The hostname or IP address of the OmenDB server
            port: The port number of the OmenDB server
        """
        self.base_url = f"http://{host}:{port}/v1"
        self.logger = logging.getLogger("OmenDBClient")
        self.logger.info(f"Initialized REST client at {self.base_url}")

    def insert(
        self, id: str, vector: List[float], metadata: Optional[Dict[str, str]] = None
    ) -> Tuple[bool, str]:
        """
        Insert a vector into the database.

        Args:
            id: Unique identifier for the vector
            vector: The vector values as a list of floats
            metadata: Optional metadata as a dictionary of string key-value pairs

        Returns:
            A tuple containing (success, message)
        """
        if metadata is None:
            metadata = {}

        self.logger.debug(f"Inserting vector {id} with dimension {len(vector)}")

        payload = {"id": id, "vector": vector, "metadata": metadata}

        try:
            response = requests.post(f"{self.base_url}/vectors", json=payload)
            response.raise_for_status()
            result = response.json()
            return result.get("success", False), result.get("message", "")
        except requests.exceptions.RequestException as e:
            self.logger.error(f"Insert request failed: {e}")
            return False, f"Request failed: {str(e)}"

    def get(
        self, id: str, include_vector: bool = True, include_metadata: bool = True
    ) -> Optional[Dict]:
        """
        Retrieve a vector from the database by ID.

        Args:
            id: The ID of the vector to retrieve
            include_vector: Whether to include the vector values in the response
            include_metadata: Whether to include metadata in the response

        Returns:
            A dictionary containing the vector information, or None if not found
        """
        self.logger.debug(f"Getting vector {id}")

        params = []
        if include_vector:
            params.append("include_vector=true")
        if include_metadata:
            params.append("include_metadata=true")

        query_string = "&".join(params)
        url = f"{self.base_url}/vectors/{id}"
        if query_string:
            url += f"?{query_string}"

        try:
            response = requests.get(url)
            if response.status_code == 404:
                return None

            response.raise_for_status()
            data = response.json()

            if not data.get("success", False):
                return None

            return data
        except requests.exceptions.RequestException as e:
            self.logger.error(f"Get request failed: {e}")
            return None

    def update(
        self,
        id: str,
        vector: Optional[List[float]] = None,
        metadata: Optional[Dict[str, str]] = None,
    ) -> Tuple[bool, str]:
        """
        Update a vector in the database.

        Args:
            id: The ID of the vector to update
            vector: Optional new vector values (if None, the existing vector is preserved)
            metadata: Optional new metadata (if None, the existing metadata is preserved)

        Returns:
            A tuple containing (success, message)
        """
        self.logger.debug(f"Updating vector {id}")

        payload = {}
        if vector is not None:
            payload["vector"] = vector
        if metadata is not None:
            payload["metadata"] = metadata

        if not payload:
            return False, "No update data provided"

        try:
            response = requests.put(f"{self.base_url}/vectors/{id}", json=payload)
            response.raise_for_status()
            result = response.json()
            return result.get("success", False), result.get("message", "")
        except requests.exceptions.RequestException as e:
            self.logger.error(f"Update request failed: {e}")
            return False, f"Request failed: {str(e)}"

    def delete(self, id: str) -> Tuple[bool, str]:
        """
        Delete a vector from the database.

        Args:
            id: The ID of the vector to delete

        Returns:
            A tuple containing (success, message)
        """
        self.logger.debug(f"Deleting vector {id}")

        try:
            response = requests.delete(f"{self.base_url}/vectors/{id}")
            response.raise_for_status()
            result = response.json()
            return result.get("success", False), result.get("message", "")
        except requests.exceptions.RequestException as e:
            self.logger.error(f"Delete request failed: {e}")
            return False, f"Request failed: {str(e)}"

    def search(
        self,
        vector: List[float],
        k: int = 10,
        include_vectors: bool = False,
        include_metadata: bool = True,
        include_distances: bool = True,
        filters: Optional[List[Dict]] = None,
    ) -> List[Dict]:
        """
        Search for the k-nearest neighbors of a query vector.

        Args:
            vector: The query vector as a list of floats
            k: The number of nearest neighbors to retrieve
            include_vectors: Whether to include vector values in the results
            include_metadata: Whether to include metadata in the results
            include_distances: Whether to include distances in the results
            filters: Optional list of metadata filters, each as a dict with "field", "operator", "value"

        Returns:
            A list of dictionaries containing the search results
        """
        self.logger.debug(f"Searching for {k} vectors similar to query vector")

        payload = {
            "vector": vector,
            "k": k,
            "include_vectors": include_vectors,
            "include_metadata": include_metadata,
            "include_distances": include_distances,
        }

        if filters:
            payload["filters"] = filters

        try:
            response = requests.post(f"{self.base_url}/search", json=payload)
            response.raise_for_status()
            data = response.json()

            if not data.get("success", False):
                return []

            return data.get("results", [])
        except requests.exceptions.RequestException as e:
            self.logger.error(f"Search request failed: {e}")
            return []

    def batch_insert(self, items: List[Dict]) -> Dict:
        """
        Insert multiple vectors in a single request.

        Args:
            items: A list of dictionaries, each containing:
                  - id: Unique identifier for the vector
                  - vector: The vector values as a list of floats
                  - metadata: Optional metadata dictionary

        Returns:
            A dictionary with the insertion results
        """
        self.logger.debug(f"Batch inserting {len(items)} vectors")

        payload = {"vectors": items}

        try:
            response = requests.post(f"{self.base_url}/vectors/batch", json=payload)
            response.raise_for_status()
            return response.json()
        except requests.exceptions.RequestException as e:
            self.logger.error(f"Batch insert request failed: {e}")
            return {
                "successful_count": 0,
                "failed_count": len(items),
                "message": f"Request failed: {str(e)}",
            }

    def count(self, filters: Optional[List[Dict]] = None) -> int:
        """
        Count vectors in the database.

        Args:
            filters: Optional list of metadata filters, each as a dict with "field", "operator", "value"

        Returns:
            The number of vectors matching the filters
        """
        self.logger.debug(f"Counting vectors" + (" with filters" if filters else ""))

        url = f"{self.base_url}/vectors/count"

        if filters:
            # In a real implementation, this would properly encode filters as query parameters
            self.logger.warning(
                "Filters in count request not yet implemented for REST API"
            )

        try:
            response = requests.get(url)
            response.raise_for_status()
            data = response.json()
            return data.get("count", 0)
        except requests.exceptions.RequestException as e:
            self.logger.error(f"Count request failed: {e}")
            return 0
