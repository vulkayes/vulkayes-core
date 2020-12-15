## Common

* vkCmdExecuteCommands
* vkCmdSetDeviceMask
* vkCmdWaitEvents
* vkCmdPipelineBarrier

* ~~vkCmdBindPipeline~~
* vkCmdBindPipelineShaderGroupNV
* vkCmdBindDescriptorSets
* vkCmdPushDescriptorSetKHR
* vkCmdPushDescriptorSetWithTemplateKHR
* vkCmdPushConstants

* vkCmdBeginQuery
* vkCmdBeginQueryIndexedEXT
* vkCmdEndQuery
* vkCmdEndQueryIndexedEXT

* vkCmdWriteTimestamp
* vkCmdSetPerformanceMarkerINTEL
* vkCmdSetPerformanceStreamMarkerINTEL
* vkCmdSetPerformanceOverrideINTEL
* vkCmdWriteBufferMarkerAMD

* vkCmdSetPrimitiveTopologyEXT

* ~~vkCmdBindIndexBuffer~~
* ~~vkCmdBindVertexBuffers~~
* vkCmdBindVertexBuffers2EXT

* vkCmdBeginConditionalRenderingEXT
* vkCmdEndConditionalRenderingEXT

* vkCmdBindTransformFeedbackBuffersEXT

* vkCmdSetViewportWScalingNV
* vkCmdSetViewportWithCountEXT
* vkCmdSetScissorWithCountEXT
* vkCmdSetViewport

* vkCmdSetSampleLocationsEXT
* vkCmdBindShadingRateImageNV
* vkCmdSetViewportShadingRatePaletteNV
* vkCmdSetCoarseSampleOrderNV

* vkCmdSetLineWidth
* vkCmdSetLineStippleEXT
* vkCmdSetFrontFaceEXT
* vkCmdSetCullModeEXT
* vkCmdSetDepthBias
* vkCmdSetDiscardRectangleEXT
* vkCmdSetScissor
* vkCmdSetExclusiveScissorNV
* vkCmdSetDepthBoundsTestEnableEXT
* vkCmdSetDepthBounds
* vkCmdSetStencilTestEnableEXT
* vkCmdSetStencilOpEXT
* vkCmdSetStencilCompareMask
* vkCmdSetStencilWriteMask
* vkCmdSetStencilReference
* vkCmdSetDepthTestEnableEXT
* vkCmdSetDepthCompareOpEXT
* vkCmdSetDepthWriteEnableEXT
* vkCmdSetBlendConstants
* vkCmdBeginDebugUtilsLabelEXT
* vkCmdEndDebugUtilsLabelEXT
* vkCmdInsertDebugUtilsLabelEXT
* vkCmdDebugMarkerBeginEXT
* vkCmdDebugMarkerEndEXT
* vkCmdDebugMarkerInsertEXT
* vkCmdSetCheckpointNV

## Outside render pass

* vkCmdPipelineBarrier - dependent
* vkCmdEndConditionalRenderingEXT - dependent

* vkCmdSetEvent
* vkCmdResetEvent
​
* ~~vkCmdBeginRenderPass~~
* vkCmdBeginRenderPass2​

* vkCmdResetQueryPool
* vkCmdCopyQueryPoolResults

* vkCmdClearColorImage
* vkCmdClearDepthStencilImage
* vkCmdFillBuffer
* vkCmdUpdateBuffer

* vkCmdCopyBuffer
* vkCmdCopyImage
* vkCmdCopyBufferToImage
* vkCmdCopyImageToBuffer
* vkCmdBlitImage
* vkCmdResolveImage

* vkCmdDispatch
* vkCmdDispatchIndirect
* vkCmdDispatchBase
* vkCmdPreprocessGeneratedCommandsNV

* vkCmdTraceRaysNV
* vkCmdTraceRaysKHR
* vkCmdTraceRaysIndirectKHR
* vkCmdBuildAccelerationStructureNV
* vkCmdBuildAccelerationStructureKHR
* vkCmdBuildAccelerationStructureIndirectKHR
* vkCmdWriteAccelerationStructuresPropertiesKHR
* vkCmdCopyAccelerationStructureNV
* vkCmdCopyAccelerationStructureKHR
* vkCmdCopyAccelerationStructureToMemoryKHR
* vkCmdCopyMemoryToAccelerationStructureKHR

## Inside render pass

* ~~vkCmdNextSubpass~~
* vkCmdNextSubpass2
* ~~vkCmdEndRenderPass~~
* vkCmdEndRenderPass2

* vkCmdClearAttachments

* vkCmdDraw
* vkCmdDrawIndexed

* vkCmdDrawIndirect
* vkCmdDrawIndirectCount
* vkCmdDrawIndexedIndirect
* vkCmdDrawIndexedIndirectCount
* vkCmdDrawIndirectByteCountEXT

* vkCmdDrawMeshTasksNV
* vkCmdDrawMeshTasksIndirectNV
* vkCmdDrawMeshTasksIndirectCountNV

* vkCmdBeginTransformFeedbackEXT
* vkCmdEndTransformFeedbackEXT
* vkCmdExecuteGeneratedCommandsNV
