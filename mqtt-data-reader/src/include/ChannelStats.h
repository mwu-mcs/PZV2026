#pragma once

#include <cstdint>
#include <string>
#include <ctime>

namespace common
{
   enum class EProbableCause
   {
      NoFault = 0,
      Misconfiguration = 1,
      EndpointRestart = 2,
      NetworkIssues = 3,
      AuthenticationFailure = 4,
      Unknown = 5,
   };

   enum class EConnectionState
   {
      Disconnected = 0,
      Connecting = 1,
      Connected = 2,
      ConnectedButIncomplete = 3,
      ConnectionLostRetrying = 4,
      ConnectionLost = 5,
   };

   struct ChannelStats
   {
      uint32_t SuccessfulReadsSinceOpen;
      uint32_t FailedReadsSinceOpen;
      double AverageReadDurationMs;
      std::string LastErrorMessage;
      EProbableCause LastProbableCause;
      EConnectionState ConnectionState;
      std::time_t LastSuccessfulReadTime;
      std::time_t LastFailedReadTime;
      std::time_t SuccessfulOpenTime;
      std::time_t FailedOpenTime;
   };

   inline void connecting(ChannelStats& stats)
   {
      stats.ConnectionState = EConnectionState::Connecting;
   }

   inline void connected(ChannelStats& stats)
   {
      stats.ConnectionState = EConnectionState::Connected;
      stats.LastErrorMessage.clear();
      stats.LastProbableCause = EProbableCause::NoFault;
      stats.SuccessfulOpenTime = std::time(nullptr);
      stats.SuccessfulReadsSinceOpen = 0;
      stats.FailedReadsSinceOpen = 0;
   }

   inline void connectionLost(ChannelStats& stats, const std::string& errorMessage, EProbableCause probableCause)
   {
      EProbableCause cause = EProbableCause::Misconfiguration;
      if(stats.ConnectionState == EConnectionState::Connected)
      {
         auto beenConnectedFor = std::difftime(std::time(nullptr), stats.SuccessfulOpenTime);
         cause = EProbableCause::EndpointRestart;
         if(beenConnectedFor < 60)
         {
            cause = EProbableCause::NetworkIssues;
         }
         stats.ConnectionState = EConnectionState::ConnectionLostRetrying;
      }
      else
      {
         cause = EProbableCause::Misconfiguration;
         stats.ConnectionState = EConnectionState::ConnectionLost;
      }

      if(probableCause == EProbableCause::Unknown)
      {
         probableCause = cause;
      }

      stats.ConnectionState = EConnectionState::ConnectionLost;
      stats.LastErrorMessage = errorMessage;
      stats.LastProbableCause = probableCause;
      stats.LastFailedReadTime = std::time(nullptr);
   }

   inline void cannotConnect(ChannelStats& stats, const std::string& errorMessage, EProbableCause probableCause = EProbableCause::Misconfiguration)
   {
      stats.ConnectionState = EConnectionState::Disconnected;
      stats.LastErrorMessage = errorMessage;
      stats.LastProbableCause =
          EProbableCause::Unknown == probableCause ? EProbableCause::Misconfiguration : probableCause;
      stats.FailedOpenTime = std::time(nullptr);
   }

   inline void successfulRead(ChannelStats& stats, double readDurationMs)
   {
      stats.SuccessfulReadsSinceOpen++;
      stats.AverageReadDurationMs =
          (stats.AverageReadDurationMs * (stats.SuccessfulReadsSinceOpen - 1) + readDurationMs) /
          stats.SuccessfulReadsSinceOpen;
      stats.LastSuccessfulReadTime = std::time(nullptr);
   }

   inline void failedRead(ChannelStats& stats, const std::string& errorMessage, EProbableCause probableCause)
   {
      stats.FailedReadsSinceOpen++;
      stats.LastErrorMessage = errorMessage;
      stats.LastProbableCause = probableCause;
      stats.LastFailedReadTime = std::time(nullptr);
   }

   inline void disconnected(ChannelStats& stats)
   {
      stats.ConnectionState = EConnectionState::Disconnected;
      stats.LastErrorMessage.clear();
      stats.LastProbableCause = EProbableCause::NoFault;
      stats.SuccessfulOpenTime = 0;
      stats.LastSuccessfulReadTime = 0;
      stats.LastFailedReadTime = 0;
      stats.SuccessfulReadsSinceOpen = 0;
      stats.FailedReadsSinceOpen = 0;
   }

   inline void reset(ChannelStats& stats)
   {
      stats = ChannelStats{};
   }

   inline std::string toString(EProbableCause cause)
   {
      switch(cause)
      {
         case EProbableCause::NoFault:
            return "NoFault";
         case EProbableCause::Misconfiguration:
            return "Misconfiguration";
         case EProbableCause::EndpointRestart:
            return "EndpointRestart";
         case EProbableCause::NetworkIssues:
            return "NetworkIssues";
         case EProbableCause::AuthenticationFailure:
            return "AuthenticationFailure";
         case EProbableCause::Unknown:
            return "Unknown";
         default:
            return "Invalid";
      }
   }

   inline std::string toString(EConnectionState state)
   {
      switch(state)
      {
         case EConnectionState::Disconnected:
            return "Disconnected";
         case EConnectionState::Connecting:
            return "Connecting";
         case EConnectionState::Connected:
            return "Connected";
         case EConnectionState::ConnectedButIncomplete:
            return "ConnectedButIncomplete";
         case EConnectionState::ConnectionLostRetrying:
            return "ConnectionLostRetrying";
         case EConnectionState::ConnectionLost:
            return "ConnectionLost";
         default:
            return "Invalid";
      }
   }
}
