"""
Pipeline builder for creating query processing pipelines.

This module provides a builder pattern for assembling multi-stage
query pipelines from reusable components, including support for
batch operations and transaction grouping.
"""

from collections import List, Dict, Optional
from core.vector import Vector
from storage.vector_store import VectorStore, QueryFilter
from index.hnsw_index import HnswGraph
from query.query_engine import (
    QueryEngine, QueryStage, FilterStage, VectorSearchStage, ReRankingStage
)
from query.batch_retrieval_stage import BatchRetrievalStage, BatchMetadataFilterStage
from util.logging import Logger, LogLevel

@value
struct QueryPipelineBuilder[dtype: DType]:
    """
    Builder for constructing query processing pipelines.
    
    This builder simplifies the process of creating complex query pipelines
    by providing a fluent interface for adding and configuring stages.
    
    Attributes:
        engine: The query engine being built
        index: Optional vector index for similarity search
        store: Optional vector store for retrieving records
        logger: Logger for build operations
    """
    
    var engine: QueryEngine[dtype]
    var index: Optional[HnswGraph[dtype]]
    var store: Optional[VectorStore[dtype]]
    var logger: Logger
    
    fn __init__(out self):
        self.engine = QueryEngine[dtype]()
        self.index = None
        self.store = None
        self.logger = Logger("QueryPipelineBuilder", LogLevel.INFO)
    
    fn __copyinit__(out self, other: Self):
        self.engine = other.engine
        self.index = other.index
        self.store = other.store
        self.logger = other.logger
    
    fn with_index(mut self, index: HnswGraph[dtype]) -> Self:
        """Set the vector index for similarity search."""
        self.index = Optional[HnswGraph[dtype]](index)
        return self
    
    fn with_store(mut self, store: VectorStore[dtype]) -> Self:
        """Set the vector store for retrieving records."""
        self.store = Optional[VectorStore[dtype]](store)
        return self
    
    fn add_vector_search(mut self, options: Dict[String, String] = Dict[String, String]()) raises -> Self:
        """
        Add a vector similarity search stage to the pipeline.
        
        Args:
            options: Configuration options for the search stage
            
        Returns:
            Self for method chaining
            
        Raises:
            Error: If index or store is not set
        """
        if self.index.none() or self.store.none():
            raise Error("Vector search requires both index and store to be set")
            
        var stage = VectorSearchStage[dtype](self.index.value(), self.store.value())
        
        if len(options) > 0:
            stage.configure(options)
            
        self.engine.add_stage(stage)
        return self
    
    fn add_filter(mut self, options: Dict[String, String] = Dict[String, String]()) -> Self:
        """
        Add a metadata filtering stage to the pipeline.
        
        Args:
            options: Configuration options for the filter stage
            
        Returns:
            Self for method chaining
        """
        var stage = FilterStage[dtype]()
        
        if len(options) > 0:
            try:
                stage.configure(options)
            except:
                self.logger.warning("Failed to configure filter stage with options")
                
        self.engine.add_stage(stage)
        return self
    
    fn add_batch_filter(mut self, options: Dict[String, String] = Dict[String, String]()) raises -> Self:
        """
        Add a batch-aware metadata filtering stage to the pipeline.
        This stage leverages batch operations for improved filtering performance.
        
        Args:
            options: Configuration options for the batch filter stage
            
        Returns:
            Self for method chaining
            
        Raises:
            Error: If store is not set
        """
        if self.store.none():
            raise Error("Batch filter requires store to be set")
            
        var stage = BatchMetadataFilterStage[dtype](self.store.value())
        
        if len(options) > 0:
            try:
                stage.configure(options)
            except:
                self.logger.warning("Failed to configure batch filter stage with options")
                
        self.engine.add_stage(stage)
        return self
    
    fn add_batch_retrieval(mut self, options: Dict[String, String] = Dict[String, String]()) raises -> Self:
        """
        Add a batch retrieval stage to the pipeline.
        This stage uses batch operations to efficiently retrieve vectors from storage.
        
        Args:
            options: Configuration options for the batch retrieval stage
            
        Returns:
            Self for method chaining
            
        Raises:
            Error: If store is not set
        """
        if self.store.none():
            raise Error("Batch retrieval requires store to be set")
            
        var stage = BatchRetrievalStage[dtype](self.store.value())
        
        if len(options) > 0:
            try:
                stage.configure(options)
            except:
                self.logger.warning("Failed to configure batch retrieval stage with options")
                
        self.engine.add_stage(stage)
        return self
    
    fn add_reranking(mut self, options: Dict[String, String] = Dict[String, String]()) -> Self:
        """
        Add a results re-ranking stage to the pipeline.
        
        Args:
            options: Configuration options for the re-ranking stage
            
        Returns:
            Self for method chaining
        """
        var stage = ReRankingStage[dtype]()
        
        if len(options) > 0:
            try:
                stage.configure(options)
            except:
                self.logger.warning("Failed to configure re-ranking stage with options")
                
        self.engine.add_stage(stage)
        return self
    
    fn add_custom_stage(mut self, stage: QueryStage[dtype]) -> Self:
        """
        Add a custom query stage to the pipeline.
        
        Args:
            stage: Custom query stage to add
            
        Returns:
            Self for method chaining
        """
        self.engine.add_stage(stage)
        return self
    
    fn build(self) raises -> QueryEngine[dtype]:
        """
        Build and return the configured query engine.
        
        Returns:
            Fully configured query engine
            
        Raises:
            Error: If the pipeline has no stages
        """
        if len(self.engine.stages) == 0:
            raise Error("Query pipeline must have at least one stage")
            
        self.logger.debug(
            "Built query pipeline with " + 
            String(len(self.engine.stages)) + " stages"
        )
            
        return self.engine