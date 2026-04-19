// \HEADER\---------------------------------------------------------------------
//  All rights reserved to mcs software ag, Switzerland
//  UTF-8 with BOM, do not save with codepage (દ೧ᥴøđĭἤց)
// -----------------------------------------------------------------------------

#pragma once

#include <inttypes.h>
#include <string>

#include "DataType.h"

namespace common
{
   union VariantData
   {
      bool Bool;
      int8_t Int8;
      int16_t Int16;
      int32_t Int32;
      int64_t Int64;
      uint8_t UInt8;
      uint16_t UInt16;
      uint32_t UInt32;
      uint64_t UInt64;
      float Float32;
      double Float64;
   };

   /**
    * @brief The Variant class contains a value of a specific data type.
    *
    */
   class Variant
   {
   public:
      DataType Type;
      VariantData Data;
      std::string String;

      Variant() = default;
      Variant(const Variant& other) = default;
      Variant(Variant&& other) noexcept = default;
      Variant& operator=(const Variant& other) = default;
      Variant& operator=(Variant&& other) noexcept = default;

      /**
       * @brief Construct a new Variant object
       *
       * @param type Data type of the value.
       */
      explicit Variant(bool value);

      /**
       * @brief Construct a new Variant object
       *
       * @param type Data type of the value.
       */
      explicit Variant(int8_t value);

      /**
       * @brief Construct a new Variant object
       *
       * @param type Data type of the value.
       */
      explicit Variant(int16_t value);

      /**
       * @brief Construct a new Variant object
       *
       * @param type Data type of the value.
       */
      explicit Variant(int32_t value);

      /**
       * @brief Construct a new Variant object
       *
       * @param type Data type of the value.
       */
      explicit Variant(int64_t value);

      /**
       * @brief Construct a new Variant object
       *
       * @param type Data type of the value.
       */
      explicit Variant(uint8_t value);

      /**
       * @brief Construct a new Variant object
       *
       * @param type Data type of the value.
       */
      explicit Variant(uint16_t value);

      /**
       * @brief Construct a new Variant object
       *
       * @param type Data type of the value.
       */
      explicit Variant(uint32_t value);

      /**
       * @brief Construct a new Variant object
       *
       * @param type Data type of the value.
       */
      explicit Variant(uint64_t value);

      /**
       * @brief Construct a new Variant object
       *
       * @param type Data type of the value.
       */
      explicit Variant(float value);

      /**
       * @brief Construct a new Variant object
       *
       * @param type Data type of the value.
       */
      explicit Variant(double value);

      /**
       * @brief Construct a new Variant object
       *
       * @param type Data type of the value.
       */
      explicit Variant(std::string value);

      /**
       * @brief Construct a new Variant object with default value
       *
       * @param type Data type of the value.
       */
      explicit Variant(DataType dataType);

      /**
       * @brief Sets the value of the Variant object to the specified value.
       * Before setting, a type conversion is performed, such that after the set,
       * the data type of the this-Variant remains the same, but the value is adopted.
       *
       * @param other
       * @return Variant&
       */
      Variant& setValue(const Variant& other);

      /**
       * @brief Tries to convert the data type of the contained value to the specified type. Returns a new Variant
       * object with the converted value. Throws on failure.
       *
       * @param type New type of the value.
       * @return Variant New Variant object with the converted value.
       */
      Variant as(DataType type) const;

      /**
       * @brief Tries to convert the data type of the contained value to the specified type. Returns true on success and
       * stores the converted value in the result parameter. Returns false on failure.
       *
       * @param type New type of the value.
       * @param result Converted value.
       * @return true Conversion was successful.
       * @return false Conversion failed.
       */
      bool tryAs(DataType type, Variant& result) const;

      /**
       * @brief Checks if the data type of the value can be converted to the specified type.
       *
       * @param type New type of the value.
       * @return true Conversion is possible.
       * @return false Conversion is not possible.
       */
      bool canConvertTo(DataType type) const;

      /**
       * @brief Get the value of the Variant object as a bool. Throws if the data type cannot be converted to bool.
       *
       * @return bool Value of the Variant object.
       */
      operator bool() const;

      /**
       * @brief Get the value of the Variant object as an int8_t. Throws if the data type cannot be converted to int8_t.
       *
       * @return int8_t Value of the Variant object.
       */
      operator int8_t() const;

      /**
       * @brief Get the value of the Variant object as an int16_t. Throws if the data type cannot be converted to
       * int16_t.
       *
       * @return int16_t Value of the Variant object.
       */
      operator int16_t() const;

      /**
       * @brief Get the value of the Variant object as an int32_t. Throws if the data type cannot be converted to
       * int32_t.
       *
       * @return int32_t Value of the Variant object.
       */
      operator int32_t() const;

      /**
       * @brief Get the value of the Variant object as an int64_t. Throws if the data type cannot be converted to
       * int64_t.
       *
       * @return int64_t Value of the Variant object.
       */
      operator int64_t() const;

      /**
       * @brief Get the value of the Variant object as a uint8_t. Throws if the data type cannot be converted to
       * uint8_t.
       *
       * @return uint8_t Value of the Variant object.
       */
      operator uint8_t() const;

      /**
       * @brief Get the value of the Variant object as a uint16_t. Throws if the data type cannot be converted to
       * uint16_t.
       *
       * @return uint16_t Value of the Variant object.
       */
      operator uint16_t() const;

      /**
       * @brief Get the value of the Variant object as a uint32_t. Throws if the data type cannot be converted to
       * uint32_t.
       *
       * @return uint32_t Value of the Variant object.
       */
      operator uint32_t() const;

      /**
       * @brief Get the value of the Variant object as a uint64_t. Throws if the data type cannot be converted to
       * uint64_t.
       *
       * @return uint64_t Value of the Variant object.
       */
      operator uint64_t() const;

      /**
       * @brief Get the value of the Variant object as a float. Throws if the data type cannot be converted to float.
       *
       * @return float Value of the Variant object.
       */
      operator float() const;

      /**
       * @brief Get the value of the Variant object as a double. Throws if the data type cannot be converted to double.
       *
       * @return double Value of the Variant object.
       */
      operator double() const;

      /**
       * @brief Gets the contents of the Variant object as a string.
       *
       * @return std::string Contents of the Variant object.
       */
      operator std::string() const;

      bool isNumeric() const;
      double toDouble() const;
      std::string toString() const;

      bool operator==(const Variant& other) const;
      bool operator!=(const Variant& other) const;
   };
}
