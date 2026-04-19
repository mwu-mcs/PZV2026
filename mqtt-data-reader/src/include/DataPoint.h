// \HEADER\---------------------------------------------------------------------
//  All rights reserved to mcs software ag, Switzerland
//  UTF-8 with BOM, do not save with codepage (દ೧ᥴøđĭἤց)
// -----------------------------------------------------------------------------

#pragma once

#include "Variant.h"
#include <ctime>
#include <string>

namespace common
{

   /**
    * @brief DataPoint class is a container for a single data point. It contains both, the meta info and the last read
    * value.
    *
    */
   class DataPoint
   {
   public:
      DataPoint() = default;
      DataPoint(const DataPoint& other) = default;
      DataPoint(DataPoint&& other) noexcept = default;
      DataPoint& operator=(const DataPoint& other) = default;
      DataPoint& operator=(DataPoint&& other) noexcept = default;
      virtual ~DataPoint() = default;

      /**
       * @brief Set the Path value. This is the controller address of the data point.
       *
       * @param path Controller address of the data point.
       */
      void setPath(const std::string& path);

      /**
       * @brief Set the Name value. This is the internal name of the data point.
       *
       * @param name Internal name of the data point.
       */
      void setName(const std::string& name);

      /**
       * @brief Get the Path value. This is the controller address of the data point.
       *
       * @return std::string Controller address of the data point.
       */
      std::string getPath() const;

      /**
       * @brief Get the internal Name. This is the internal name of the data point.
       *
       * @return std::string Internal name of the data point.
       */
      std::string getName() const;

      /**
       * @brief Set the Element Count value. This is the number of elements in the data point. Must be equal or greather
       * than 1.
       *
       * @param elementCount Number of elements in the data point.
       */
      void setElementCount(unsigned short elementCount);

      /**
       * @brief Get the Element Count value. This is the number of elements in the data point. Must be equal or greather
       * than 1.
       *
       * @return unsigned short Number of elements in the data point.
       */
      unsigned short getElementCount() const;

      /**
       * @brief Set the data type of the value.
       *
       * @param type Data type of the data point.
       */
      void setType(DataType type);

      /**
       * @brief Get the data type of the value.
       *
       * @return DataType Data type of the data point.
       */
      DataType getType() const;

      /**
       * @brief Set the the time when the data point was last updated.
       *
       * @param timeStamp Time when the data point was last updated.
       */
      void setTimeStamp(std::time_t timeStamp);

      /**
       * @brief Get the the time when the data point was last updated.
       *
       * @return std::time_t Time when the data point was last updated.
       */
      std::time_t getTimeStamp() const;

      /**
       * @brief Set the value of the data point.
       *
       * @param value Value of the data point.
       */
      void setValue(const Variant& value);

      /**
       * @brief Get the value of the data point.
       *
       * @return Variant Value of the data point.
       */
      Variant getValue() const;

      /**
       * @brief Get the reference on the value of the data point for modifications etc.
       *
       * @return Variant& Value of the data point.
       */
      Variant& getValueRef();

      /**
       * @brief Set the datasource from which the data point was read.
       *
       * @param updatedBy Datasource from which the data point was read.
       */
      void setUpdatedBy(const std::string& updatedBy);

      /**
       * @brief Get the datasource from which the data point was read.
       *
       * @return std::string Datasource from which the data point was read.
       */
      std::string getUpdatedBy() const;

      bool operator==(const DataPoint& other) const;
      bool operator!=(const DataPoint& other) const;

   private:
      std::string m_strName;
      std::string m_strPath;
      unsigned short m_usElementCount = 1;
      DataType m_eType = DataType::Int16;
      std::time_t m_tTimeStamp = 0;
      Variant m_vValue;
      std::string m_strUpdatedBy;
   };
}
