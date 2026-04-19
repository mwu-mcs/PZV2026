// \HEADER\---------------------------------------------------------------------
//  All rights reserved to mcs software ag, Switzerland
//  UTF-8 with BOM, do not save with codepage (દ೧ᥴøđĭἤց)
// -----------------------------------------------------------------------------

#pragma once

#include <inttypes.h>
#include <string>
#include <algorithm>

namespace common
{
   enum class DataType
   {
      Bool,
      Int8,
      Int16,
      Int32,
      Int64,
      UInt8,
      UInt16,
      UInt32,
      UInt64,
      Float32,
      Float64,
      String,
      Unspecified
   };

   namespace dataType
   {
      /**
       * @brief Calculates the corresponding DataType enum value for a template argument. Creates a compile-time error
       * if the type is not supported.
       *
       * @tparam T DataType to test
       * @return Matching DataType enum
       */
      template<typename T>
      struct from_impl;

      template<>
      struct from_impl<bool>
      {
         static const DataType value = DataType::Bool;
      };

      template<>
      struct from_impl<int8_t>
      {
         static const DataType value = DataType::Int8;
      };

      template<>
      struct from_impl<int16_t>
      {
         static const DataType value = DataType::Int16;
      };

      template<>
      struct from_impl<int32_t>
      {
         static const DataType value = DataType::Int32;
      };

      template<>
      struct from_impl<int64_t>
      {
         static const DataType value = DataType::Int64;
      };

      template<>
      struct from_impl<uint8_t>
      {
         static const DataType value = DataType::UInt8;
      };

      template<>
      struct from_impl<uint16_t>
      {
         static const DataType value = DataType::UInt16;
      };

      template<>
      struct from_impl<uint32_t>
      {
         static const DataType value = DataType::UInt32;
      };

      template<>
      struct from_impl<uint64_t>
      {
         static const DataType value = DataType::UInt64;
      };

      template<>
      struct from_impl<float>
      {
         static const DataType value = DataType::Float32;
      };

      template<>
      struct from_impl<double>
      {
         static const DataType value = DataType::Float64;
      };

	  template<>
      struct from_impl<std::string>
      {
         static const DataType value = DataType::String;
      };


      template<typename T>
      DataType from()
      {
         return from_impl<T>::value;
      }

      inline DataType parse(const std::string& str)
      {
         std::string lowerStr;
         lowerStr.resize(str.size());
         std::transform(str.begin(), str.end(), lowerStr.begin(), ::tolower);

         if(lowerStr == "bool" || lowerStr == "boolean")
         {
            return DataType::Bool;
         }
         else if(lowerStr == "int8" || lowerStr == "char" || lowerStr == "i8")
         {
            return DataType::Int8;
         }
         else if(lowerStr == "int16" || lowerStr == "short" || lowerStr == "i16")
         {
            return DataType::Int16;
         }
         else if(lowerStr == "int32" || lowerStr == "int" || lowerStr == "long" || lowerStr == "i32")
         {
            return DataType::Int32;
         }
         else if(lowerStr == "int64" || lowerStr == "longlong" || lowerStr == "long long" || lowerStr == "i64")
         {
            return DataType::Int64;
         }
         else if(lowerStr == "uint8" || lowerStr == "uchar" || lowerStr == "byte" || lowerStr == "u8")
         {
            return DataType::UInt8;
         }
         else if(lowerStr == "uint16" || lowerStr == "ushort" || lowerStr == "u16")
         {
            return DataType::UInt16;
         }
         else if(lowerStr == "uint32" || lowerStr == "uint" || lowerStr == "ulong" || lowerStr == "u32")
         {
            return DataType::UInt32;
         }
         else if(lowerStr == "uint64" || lowerStr == "ulonglong" || lowerStr == "unsigned long long" || lowerStr == "u64")
         {
            return DataType::UInt64;
         }
         else if(lowerStr == "float" || lowerStr == "float32" || lowerStr == "f32")
         {
            return DataType::Float32;
         }
         else if(lowerStr == "double" || lowerStr == "float64" || lowerStr == "f64")
         {
            return DataType::Float64;
         }
         else if(lowerStr == "string" || lowerStr == "str")
         {
            return DataType::String;
         }

         return DataType::Unspecified;
      }
   }
}
