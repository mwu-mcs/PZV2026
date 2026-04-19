// \HEADER\---------------------------------------------------------------------
//  All rights reserved to mcs software ag, Switzerland
//  UTF-8 with BOM, do not save with codepage (દ೧ᥴøđĭἤց)
// -----------------------------------------------------------------------------

#pragma once

#include "DataPoint.h"
#include "ChannelStats.h"
#include <vector>
#include <string>

namespace common
{
   /**
    * @brief IDataSource is an interface for data sources. It provides methods to open and close the data source and to
    * read data points from it.
    *
    */
   class IDataSource
   {
   public:
      virtual ~IDataSource() = default;

      virtual void open() = 0;
      virtual void close() = 0;
      virtual bool isOpen() const = 0;

      virtual bool readAll() = 0;
      virtual bool get(const std::string& name, DataPoint& o_rValue) = 0;

      virtual std::string getId() const = 0;
      virtual const ChannelStats& getStats() const = 0;
   };
}
